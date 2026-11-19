# Research — Product Name Shortlist (OQ-001)

> **Status:** Draft, awaiting choice · **Last updated:** 2026-11-19
> **Related:** [open-questions OQ-001](../open-questions.md) · [decisions](../decisions.md)

## Brief

The name should evoke **study, mastery, autonomy and scholarship**, and suit the audience path: university students → postgraduates → lifelong academics (D-018). The product owner suggested something along the lines of *Scholar*.

## Criteria

1. Meaning: fits study / mastery / autonomy / scholarship, and ages well from undergrad to professor.
2. Works as a **CLI binary**: short, lowercase, easy to type, no clash with common commands.
3. **Searchable**: not drowned out by a dominant existing product.
4. **Available**: crate/package names, domains, app-store names, and no conflicting trademarks in software/education (Nice classes 9, 41, 42).
5. Pronounceable in English, and not awkward in other major languages.

## Why not plain "Scholar"

*Scholar* fits the brief, but **Google Scholar** owns that word in exactly our audience (academics, postgrads). The name would be effectively unsearchable and easy to confuse with it. Using a derivative keeps the idea while staying distinct.

## Shortlist

Package-registry snapshot taken 2026-11-19: "free" means the bare name is unclaimed. npm names can be scoped (`@name/cli`) instead, so crates.io matters more for `cargo install`. **No trademark or domain search has been done yet.**

| Name | Meaning / story | CLI | crates.io | npm | Notes / risks |
|---|---|---|---|---|---|
| **Studium** ⭐ | Latin "study, zeal, devoted effort". Also the medieval word for a university (*studium generale*), which bridges student and academic. | `studium` | free | taken | Generic Latin word; some universities may use it for internal portals. Needs a trademark check. |
| **Scholaris** ⭐ | Latin "of scholars"; keeps the *Scholar* idea while staying distinct from Google Scholar | `scholaris` (alias `sch`) | free | free | Longer to type, so ship a short alias. Closest to the original suggestion. |
| **Sapere** ⭐ | From *Sapere aude*, "dare to know": Kant's Enlightenment motto for thinking for yourself (autonomy + scholarship) | `sapere` | free | taken | Pronunciation varies (SAH-peh-reh). Name is used by some consultancies (check trademark classes). |
| Schole | Greek σχολή, "leisure devoted to learning", the root of both *school* and *scholar* | `schole` | free | free | Strong story, but ambiguous pronunciation (SKOH-lay). |
| Autodidact | "Self-taught"; strongest autonomy signal | `autodidact` | free | taken | Long. Leans "self-learner", less natural for university students. |
| Ingenium | Latin "intellect, talent, ingenuity" | `ingenium` | free | taken | Less directly about study. |
| Arete | Greek "excellence, mastery" | `arete` | taken | taken | Short and elegant, but crowded. |
| Polymath | Broad mastery; suits lifelong academics | `polymath` | taken | taken | Crowded; sounds like a maths app. |

Considered and rejected: *Magister* (widely used school-management software), *Opus* (crowded: codec, AI model names), *Praxis* (a well-known exam brand), *Mentat* (fictional-universe trademark), *Lyceum* / *Athenaeum* (crowded, long).

## Recommendation

1. **Studium.** Best fit for the uni → postgrad → academic path, short CLI, and the crate name is free.
2. **Scholaris** if you'd like to stay closest to *Scholar*. Both registry names are free.
3. **Sapere** if autonomy ("dare to know") should be the headline.

## Next steps once a name is picked

- [ ] Trademark search (USPTO, EUIPO, UKIPO) in classes 9, 41, 42
- [ ] Domains (`.app`, `.com`, `.io`), GitHub org, app-store name availability
- [ ] Reserve the crates.io name and an npm scope
- [ ] Resolve OQ-001; replace the working titles (`StudyApp`, `study`, `studyapp://`) across `docs/specs/` and `CLAUDE.md`
