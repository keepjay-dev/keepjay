# Research — Product Name, Round 4 (Full Re-evaluation)

> **Status:** Proposed, awaiting product-owner decision (OQ-001 / D-019) · **Last updated:** 2026-11-22
> **Related:** [naming.md](naming.md) (rounds 1–3, method, CLI conventions §6, pre-release checklist §7) · [decisions D-019](../decisions.md) · [developer-strategy](developer-strategy.md)

## 1. Question

The product owner doesn't like *Addisco*. Rounds 1–3 searched mostly in Latin and Greek study vocabulary, and then in memory words and programming puns. Round 4 searches **from nine different angles**, and evaluates the finalists (including the earlier favourites) on a **broader rubric** that also covers brand feel and breadth.

## 2. Short answer

| # | Name | Score | Why it's here | Main risk |
|---|---|---|---|---|
| 1 | **Muista** | 4.33 | Finnish for "remember!", which is the product's job in one word. The cleanest checks of any round: all registries free, 21 GitHub repos, no products found, `.dev`/`.sh`/`.io`/`.app` look free. | An unfamiliar foreign word, like Addisco (MOO-ees-tah). It's an everyday word in Finnish, so it's generic there. |
| 2 | **Cofio** | 4.20 | Welsh for "to remember". 5 letters, alias `cof`, all registries free, 9 GitHub repos. | Welsh says KOV-yo, English readers say "coffee-o". A *Cofio Software* company exists (founded 2006). |
| 3 | **Scrubjay** | 4.17 | An English word with a story and a mascot: the scrub jay remembers **what** it hid, **where** and **when** (Clayton & Dickinson, *Nature* 1998). That maps onto cards (what), anchors (where) and stale detection (when). `scrubjay.com` looks free. | 8 letters (alias `sj`); "scrub" is mild slang for a loser |
| 4 | Edisco | 4.05 | Latin "I learn by heart" (the round-3 leader) | "e-discovery" and "EDI" search noise |
| 5 | Knowkeep | 4.03 | Says the tagline ("keep what you know"); the GitHub handle is free | Descriptive and plain |
| 9 | Addisco | 3.95 | The current name | Unfamiliar word, flat brand feel |

**Recommendation: Scrubjay**, with **Muista** as the alternative.

- If the problem with Addisco is that it's an **obscure word**, Muista and Cofio have the same problem. Scrubjay is a real English word with a memorable story, a natural logo (a small blue bird), and the only free `.com` among the finalists. Its story describes the product's own model: what, where and when.
- If you want the **cleanest, most literal** name and don't mind a foreign word, choose Muista. It scores highest.

Run a say-aloud test with 5–10 developers during the M0.6 interviews before committing (naming.md §7).

## 3. Method

### 3.1 Angles (58 new names)

| Angle | Examples |
|---|---|
| A. Learning science | Eidetic, Eidet, Engrave, Rehearse |
| B. Preservation and craft | Ambre (amber), Patina, Strata, Kiln, Anneal, Inkstone, Lode, Larder, Mortar, Tidemark |
| C. Animals with famous memories | Scrubjay, Corvid, Tusk, Pachy (elephants), Ayumu (the chimpanzee that beat humans at memory tests), Goldfish (ironically) |
| D. Coined brand names (like Vercel, Deno, Biome) | Revi, Rekko, Remly, Kerno, Mivo, Retto, Tenko, Nemi, Memry, Recka |
| E. Developer culture | Prefetch, Readahead, Warmup, Stickybit (the Unix sticky bit) |
| F. "Remember" / "memory" in other languages | Muista and Muisti (Finnish), Minne (Swedish), Cofio (Welsh), Ingat (Indonesian/Malay), Kumbuka (Swahili), Kioku and Omoide (Japanese), Memori and Memoro (Esperanto), Pomni (Russian), Memento and Retine (Latin) |
| G. Descriptive compounds | Keepsake, Mindstash, Knowkeep, Rememory, Longmem |
| H. Myth and classics | Alethe (*a-lethe*, "un-forgetting", from Lethe, the river of forgetting), Thoth, Scholia |
| I. Repetition and rhythm | Refrain, Reprise, Encore, Spiral |

### 3.2 Checks

These are the same automated checks as round 3 (naming.md §15.2), run on 2026-11-22:

- crates.io, npm and PyPI;
- Homebrew formula and cask, and Debian packages, for both the name and its alias;
- local commands;
- GitHub repositories with the name, and the GitHub handle;
- DNS for `.com`, `.dev`, `.sh`, `.io` and `.app`.

**14 names were free on all three registries:** Knowkeep, Cofio, Muista, Scrubjay, Stickybit, Kumbuka, Recka, Rekko, Retto, Eidet, Memoro, Muisti, Retine and Ingat. The ten most promising were web-checked for existing products, plus Edisco and Addisco again.

### 3.3 Scoring (broader than rounds 1–3)

| Criterion | Weight | How it's scored |
|---|---|---|
| Uniqueness | 25% | As in naming.md §3 (registries, GitHub noise, web conflicts) |
| CLI fit | 20% | As in round 3 (length; the name or alias already a command; hand alternation) |
| Meaning and story | 15% | How well it says "keep what you learn", and whether it has a story worth telling |
| Say and spell | 15% | Spelled correctly after one hearing, by English speakers and in other major languages |
| **Brand feel** | 15% | Memorable, distinctive, works as a logo or mascot, sounds like a product people would use |
| **Breadth** | 10% | Still fits if the product grows beyond developers, and doesn't mislead (as Memheap does) |

The round-1–3 finalists (Addisco, Edisco, Memini, Lampas, Memheap) were rescored on the same rubric, so everything is directly comparable.

## 4. Results

| # | Name | Alias | Score | Uniq | CLI | Meaning | Say | Brand | Breadth | Web | GitHub | Registries taken | Domains free (DNS) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | **Muista** | `mui` | **4.33** | 5.0 | 4.0 | 4.5 | 4.0 | 4.0 | 4.0 | L | 21 | none | `.dev` `.sh` `.io` `.app` |
| 2 | **Cofio** | `cof` | **4.20** | 4.0 | 5.0 | 4.5 | 3.5 | 4.0 | 4.0 | M | 9 | none | `.dev` `.sh` `.io` |
| 3 | **Scrubjay** | `sj` | **4.17** | 4.5 | 3.5 | 4.0 | 4.5 | 4.5 | 4.0 | L | 30 | none | **`.com`** `.sh` |
| 4 | Edisco | `edi` | 4.05 | 4.0 | 5.0 | 4.5 | 3.5 | 3.0 | 4.0 | L | 235 | none | `.dev` `.sh` |
| 5 | Knowkeep | `kk` | 4.03 | 5.0 | 3.0 | 4.0 | 4.5 | 3.0 | 4.5 | L | 6 | none | `.dev` `.sh` `.io` |
| 6 | Kumbuka | `kum` | 4.03 | 3.5 | 4.0 | 4.5 | 4.5 | 4.0 | 4.0 | M | 48 | none | `.dev` `.sh` |
| 7 | Lampas | `lam` | 4.00 | 4.5 | 4.5 | 2.5 | 4.5 | 3.5 | 4.0 | L | 88 | none | `.sh` `.app` |
| 8 | Memini | `mem` | 3.97 | 3.5 | 4.5 | 4.5 | 4.0 | 3.5 | 4.0 | L | 90 | npm, PyPI | `.sh` |
| 9 | Addisco | `addi` | 3.95 | 4.5 | 4.0 | 4.0 | 3.5 | 3.0 | 4.5 | L | 39 | none | `.dev` `.sh` `.io` `.app` |
| 10 | Eidet | `eid` | 3.93 | 4.0 | 5.0 | 4.5 | 3.0 | 3.0 | 3.5 | L | 166 | none | `.sh` `.io` `.app` |
| 11 | Muisti | `mus` | 3.90 | 3.5 | 4.5 | 4.0 | 4.0 | 3.5 | 4.0 | – | 370 | none | `.dev` `.sh` |
| 12 | Memoro | `mro` | 3.87 | 3.0 | 5.0 | 4.0 | 4.5 | 3.0 | 4.0 | M | 175 | none | `.dev` `.sh` `.app` |
| 13 | Stickybit | `sb` | 3.75 | 4.5 | 2.5 | 3.5 | 5.0 | 4.0 | 2.5 | L | 36 | none | `.sh` |
| 14 | Memheap | `mh` | 3.75 | 5.0 | 4.0 | 3.0 | 4.5 | 2.5 | 2.0 | L | 2 | none | all five |
| 15 | Ingat | `ing` | 3.72 | 2.5 | 4.5 | 4.0 | 4.5 | 3.5 | 4.0 | M | 787 | none | `.dev` `.sh` |

Web conflicts found:
- **Cofio:** Cofio Software, a San Diego company founded in 2006.
- **Kumbuka:** a business-card app that sells "relationship memory".
- **Memoro:** a Google Play developer of that name, plus near-names (Memora, Memoru).
- **Ingat:** an "IngatIn" productivity app.
- **Rekko:** several current apps, which rules it out.

Stickybit's namesake, Stickybits, is a discontinued barcode platform.

## 5. Finalist profiles

### Scrubjay (`scrubjay` / `sj`)

- **Story:** the scrub jay caches food and later remembers *what* it hid, *where*, and *when*. It even learns to skip caches that would have spoiled. That was the first evidence of episodic-like memory in an animal (Clayton & Dickinson, *Nature* 395, 1998). The parallel with the product is direct: a card (what), anchored to code (where), flagged when the code changes (when).
- **Brand:** a natural mascot and logo. Friendly, distinctive, an English word. It's one compound word, so it searches well.
- **CLI:** `scrubjay` is 8 letters, but the alias `sj` is 2 and collides with nothing (`addi`-style use: `sj while -- cargo test`).
- **Risks:**
  - "scrub" is slang for someone useless (TLC's "No Scrubs"), mild and mostly American;
  - "jay" brand names are common (Jay, Blue Jay), but "Scrubjay" itself had no products;
  - the GitHub handle is taken.
- **Domains:** `scrubjay.com` and `.sh` look free.

### Muista (`muista` / `mui`)

- **Story:** Finnish imperative, "remember!". The product's job in one word, and a quiet nod to Finland's place in programming (Linux, SSH, MySQL).
- **Brand:** soft and distinctive. It suits a calm, minimalist look.
- **CLI:** 6 letters with good hand alternation; alias `mui`.
- **Risks:**
  - unfamiliar to non-Finns, the same complaint as Addisco;
  - pronunciation varies (MOO-ees-tah);
  - in Finland it's an everyday word, so it's generic there, the same penalty Studium got for German (naming.md §4);
  - the GitHub handle is taken.
- **Domains:** `.dev`, `.sh`, `.io` and `.app` look free.

### Cofio (`cofio` / `cof`)

- **Story:** Welsh, "to remember".
- **Brand:** short, friendly and warm. Many readers will hear "coffee-o", which is charming for developers but blurs the meaning.
- **CLI:** 5 letters, the best CLI fit of the finalists.
- **Risks:**
  - Cofio Software (founded 2006) exists;
  - the Welsh pronunciation (KOV-yo) differs from how English readers say it;
  - the GitHub handle is taken.
- **Domains:** `.dev`, `.sh` and `.io` look free.

## 6. What each angle taught us

- **Animals** gave the strongest brands. Most famous memory animals are taken as names (Corvid, Tusk, Goldfish, Jumbo-style), but the specific, slightly unusual species (Scrubjay) are free.
- **Other languages** gave the cleanest names. The "remember" verbs of mid-sized languages (Finnish, Welsh, Swahili) are free everywhere. The trade-off is that they're foreign words, the same thing the product owner dislikes in Addisco.
- **Preservation and craft** words (Amber, Kiln, Anneal, Patina, Strata, Lode) are lovely but taken on every registry, with thousands of GitHub repos.
- **Coined brand names** clear the registries but say nothing (Recka, Retto), or collide with existing apps (Rekko).
- **Developer culture** (Stickybit, Prefetch, Readahead) either misleads, like round 3's puns, or is too long.
- **Descriptive compounds** (Knowkeep) are clear and free, but plain. They suit a feature, not a brand.
- **Myth and classics:** Alethe (*a-lethe*, "un-forgetting") has the best story in the round, but PyPI is taken and there are 1,616 GitHub repos. Thoth is taken.

## 7. Next steps

1. **Decide which problem you want solved.** If it's that the name is an *obscure word*, choose Scrubjay. If it's that it *doesn't say what it does*, choose Muista (or Cofio).
2. **Say-aloud test:** 5–10 developers and CS students during the M0.6 interviews. Can they spell it after hearing it, and what does it make them think of?
3. **On the day you decide:**
   - register the domain (`scrubjay.com`, or `muista.dev`);
   - reserve the crate, the npm scope and a GitHub org (all three finalists' handles are taken, so use e.g. `scrubjay-dev`);
   - run the trademark search (classes 9, 41 and 42).
4. **Rename** before the 0.2 alpha. The D-019 identifier table lists everything a rename touches: display name, binary and alias, URL scheme, environment variables, data directory, file extensions and package names. After that alpha the data directory needs a migration.

## 8. Sources

- Scrub jay memory: [Clayton & Dickinson, *Nature* 1998](https://www.doi.org/10.1038/26216) · [paper PDF](https://www.cs.helsinki.fi/group/cosco/Teaching/CoscoSeminar/spring2007/articles/clayton-1998.pdf) · [2003 review](https://pmc.ncbi.nlm.nih.gov/articles/PMC1088530)
- Web checks: [Cofio Software](https://ithistory.org/node/1732) · [Kumbuka](https://kumbuka-front.pages.dev/features) · [Stickybits (discontinued)](https://alternativeto.net/software/stickybits/about) · [Memoro (Google Play)](https://play.google.com/store/apps/developer?id=Memoro) · [IngatIn](https://apps.apple.com/app/ingatin/id6757688591) · [Rekko](https://www.producthunt.com/products/rekko/makers) · [Skillkeep](https://apps.apple.com/us/app/skillkeep/id6736385259) (near Knowkeep) · [EIDA](https://sourceforge.net/software/product/EIDA-Solutions/) (near Eidet)
- Searches for Muista and Scrubjay found no products ([Muna Flashcards](https://apps.apple.com/us/app/id1568849681) and ScrubDex were the nearest names)
