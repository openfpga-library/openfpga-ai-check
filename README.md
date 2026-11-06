# openfpga-ai-check

Run a AI check on cores & host the results for all updaters to pull from

## Adding a check

- checks can be added into `src/checks`, they should implement the `Check` trait in a unit struct (see others)
- they should add a score which either adds points or removes points from the AI score

## The AI score

- Currently it's unbounded but eventually it'll be from 0 to 1
- With the intention to score a fully vibe-coded core 1, and one that's had the occasional vibe-coded PR a lower score

## Declaring AI use

- A core's author can declare AI use by listing the tools they used in `Cores/<core id>/updaters.json` inside the release zip:

```json
{
    "declare_ai": ["Claude"]
}
```

- The openFPGA library publishes the list in `api/v3/cores.json`, and `DeclarationCheck` reads it from the core's latest release
- A non-empty list scores `DeclaredAi`, which sets the overall score to 1 and overrides every other check, including `GuaranteeHuman`

## Running the check

- The app is designed to run on a scheduled github workflow, with the reports being commited back into the repo
- It'll tolerate running into the github rate limit, so long as it doesn't happen before it's finished 1 core, and build up the reports over time
- Then there'll be a second workflow to merge all the JSON files together & publish them to this repo's github pages page

## The check whitelist

- Eventually there'll be some sort of whitelist that core developers can ask to be put on if this repo's scoring them incorrectly, where each core will have a check zero'd manually
