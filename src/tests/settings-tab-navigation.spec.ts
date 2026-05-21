import { expect, test } from "@playwright/test";

import { createAppHarness, copet } from "./app-harness";

const sectionCases = [
  { section: "agents", heading: "Agent integrations" },
  { section: "preferences", heading: "General" },
  { section: "about", heading: "About" },
  { section: "pets", heading: "Pets" },
] as const;

test("navigate-to-section event activates each settings tab", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      responsePaused: false,
    },
  });

  const page = await harness.openPage("settings");
  // Default landing tab is Pets.
  await expect(page.getByRole("heading", { name: "Pets" })).toBeVisible();

  for (const { section, heading } of sectionCases) {
    await page.evaluate(
      ({ event, payload }) => {
        (
          window as unknown as {
            __copetTestEmit: (e: string, p: unknown) => void;
          }
        ).__copetTestEmit(event, payload);
      },
      { event: "copet-navigate-to-section", payload: section },
    );

    await expect(page.getByRole("heading", { name: heading })).toBeVisible();
  }
});
