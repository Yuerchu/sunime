"""Generate pinyin syllable table for sunime."""

import os

INITIALS = [
    "b", "p", "m", "f",
    "d", "t", "n", "l",
    "g", "k", "h",
    "j", "q", "x",
    "zh", "ch", "sh", "r",
    "z", "c", "s",
]

FINALS_FOR_BPMF = [
    "a", "o", "e", "i", "u",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng",
    "ia", "ie", "iao", "iu",
    "ian", "in", "iang", "ing",
]

FINALS_FOR_GKH = [
    "a", "e", "u",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng", "ong",
    "ua", "uo", "uai", "ui",
    "uan", "un", "uang",
]

FINALS_FOR_ZCS = [
    "a", "e", "i", "u",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng", "ong",
    "ua", "uo", "uai", "ui",
    "uan", "un", "uang",
]

FINALS_FOR_ZHCHSHR = FINALS_FOR_ZCS.copy()

FINALS_FOR_JQX = [
    "i", "u", "ue",
    "ia", "ie", "iao", "iu",
    "ian", "in", "iang", "ing", "iong",
    "uan", "un",
]

FINALS_FOR_DT = [
    "a", "e", "i", "u",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng", "ong",
    "ia", "ie", "iao", "iu",
    "ian", "ing",
    "uo", "ui", "uan", "un",
]

FINALS_FOR_NL = [
    "a", "e", "i", "u", "v",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng", "ong",
    "ia", "ie", "iao", "iu",
    "ian", "in", "iang", "ing",
    "uo", "uan", "un",
    "ve",
]

ZERO_INITIAL = [
    "a", "o", "e",
    "ai", "ei", "ao", "ou",
    "an", "en", "ang", "eng", "er",
]

YI_SERIES = [
    "yi", "ya", "ye", "yao", "you",
    "yan", "yin", "yang", "ying", "yong",
    "yu", "yue", "yuan", "yun",
]

WU_SERIES = [
    "wu", "wa", "wo", "wai", "wei",
    "wan", "wen", "wang", "weng",
]


def generate():
    valid = set()

    valid.update(ZERO_INITIAL)
    valid.update(YI_SERIES)
    valid.update(WU_SERIES)

    for i in ["b", "p", "m", "f"]:
        for f in FINALS_FOR_BPMF:
            valid.add(i + f)

    for i in ["d", "t"]:
        for f in FINALS_FOR_DT:
            valid.add(i + f)

    for i in ["n", "l"]:
        for f in FINALS_FOR_NL:
            valid.add(i + f)

    for i in ["g", "k", "h"]:
        for f in FINALS_FOR_GKH:
            valid.add(i + f)

    for i in ["j", "q", "x"]:
        for f in FINALS_FOR_JQX:
            valid.add(i + f)

    for i in ["zh", "ch", "sh", "r"]:
        for f in FINALS_FOR_ZHCHSHR:
            valid.add(i + f)

    for i in ["z", "c", "s"]:
        for f in FINALS_FOR_ZCS:
            valid.add(i + f)

    # special: standalone zhi/chi/shi/ri/zi/ci/si
    for s in ["zhi", "chi", "shi", "ri", "zi", "ci", "si"]:
        valid.add(s)

    # some extras commonly used
    extras = [
        "dia", "dei", "den", "lo", "me",
        "nia", "fo",
        "nun", "cen", "sen", "nen",
        "kei", "tei", "shei",
    ]
    valid.update(extras)

    return sorted(valid)


def main():
    syllables = generate()
    script_dir = os.path.dirname(os.path.abspath(__file__))
    out_path = os.path.join(
        script_dir, "..", "crates", "sunime-core", "src", "pinyin_table.txt"
    )
    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        for s in syllables:
            f.write(s + "\n")
    print(f"{len(syllables)} syllables -> {out_path}")


if __name__ == "__main__":
    main()
