use std::cell::Cell;

use windows::Win32::UI::TextServices::*;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::Foundation::*;
use windows_core::*;

use sunime_ipc::{KeyEventParams, Request, Response};
use sunime_ipc::pipe::IpcClient;

fn make_key_params(wparam: WPARAM, lparam: LPARAM) -> KeyEventParams {
    let vk = wparam.0 as u32;
    let scancode = ((lparam.0 >> 16) & 0xFF) as u32;
    let shift = unsafe { GetKeyState(0x10) < 0 };
    let ctrl = unsafe { GetKeyState(0x11) < 0 };
    let alt = unsafe { GetKeyState(0x12) < 0 };
    KeyEventParams { vk, scancode, shift, ctrl, alt }
}

#[implement(ITfTextInputProcessorEx, ITfKeyEventSink)]
pub struct TextService {
    ipc: IpcClient,
    client_id: Cell<u32>,
}

impl TextService {
    pub fn new() -> Self {
        Self {
            ipc: IpcClient::new(),
            client_id: Cell::new(0),
        }
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.client_id.set(tid);

        let tm: ITfThreadMgr = ptim.clone().unwrap();
        unsafe {
            let ks: ITfKeystrokeMgr = tm.cast()?;
            let this: IUnknown = self.to_interface();
            let sink: ITfKeyEventSink = this.cast()?;
            ks.AdviseKeyEventSink(tid, &sink, true)?;
        }

        let _ = self.ipc.send(&Request::Activate);
        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        let _ = self.ipc.send(&Request::Deactivate);
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        ITfTextInputProcessor_Impl::Activate(self, ptim, tid)
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(&self, _pic: Ref<'_, ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let params = make_key_params(wparam, lparam);
        if params.ctrl || params.alt {
            return Ok(FALSE);
        }
        match self.ipc.send(&Request::OnTestKeyDown(params)) {
            Some(Response::TestKeyReply { handled: true }) => Ok(TRUE),
            _ => Ok(FALSE),
        }
    }

    fn OnTestKeyUp(&self, _pic: Ref<'_, ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(FALSE)
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let params = make_key_params(wparam, lparam);
        if params.ctrl || params.alt {
            return Ok(FALSE);
        }

        let response = self.ipc.send(&Request::OnKeyDown(params));
        match response {
            Some(Response::KeyReply { handled: true, commit, .. }) => {
                if let Some(text) = commit {
                    let ctx: ITfContext = pic.clone().unwrap();
                    let _ = insert_text(&ctx, self.client_id.get(), &text);
                }
                Ok(TRUE)
            }
            _ => Ok(FALSE),
        }
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(FALSE)
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(FALSE)
    }
}

fn insert_text(context: &ITfContext, tid: u32, text: &str) -> Result<()> {
    let text_w: Vec<u16> = text.encode_utf16().collect();
    let session = InsertTextSession {
        context: context.clone(),
        text: text_w,
    };
    let session_itf: ITfEditSession = session.into();
    unsafe {
        let _hr = context.RequestEditSession(tid, &session_itf, TF_ES_READWRITE | TF_ES_SYNC)?;
    }
    Ok(())
}

#[implement(ITfEditSession)]
struct InsertTextSession {
    context: ITfContext,
    text: Vec<u16>,
}

impl ITfEditSession_Impl for InsertTextSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        unsafe {
            let insert: ITfInsertAtSelection = self.context.cast()?;
            let _range = insert.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &self.text)?;
        }
        Ok(())
    }
}
