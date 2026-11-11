# specs/

- `behavior/<area>/<id>-<name>.md`: behaviour specs (test steps, observations, expected
  results, open questions, CSP version, date, observation method). Status: **draft → observed → implemented**.
- `formats/<format>.md`: format findings, each tied to the sample file it came from.
- Observations are never invented. A spec stays **draft** until someone has actually run it in
  CSP 1.13.2 and recorded the results.

## Observation methods
- `automated-Wine`: Claude driving CSP 1.13.2 under Wine 11.4 in Xvfb (see
  `docs/observation/wine-harness.md`). UI, screenshots and exported files only. Pixel values are
  always read from exported PNGs, never from screenshots.
- `manual-native`: the maintainer, on native Windows/macOS, with a real pen where needed.

Results that look odd under Wine get flagged `verify-native`.

## Spec template
```
# <ID> <Title>
- Status: draft | observed | implemented
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK | maintainer + real pen
- Date / method / observer: …
## Purpose
## Setup (exact)
## Steps (exact)
## Expected result / what to measure
## Observations (only real ones; date + method + data files)
## Open questions
```
