import { expect, test } from "@playwright/test";

import { createAppHarness, copet } from "./app-harness";

test("show messages switch calls set_agent_message_visible and syncs across windows", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: true,
    },
  });

  const settingsPage = await harness.openPage("settings");
  await settingsPage.getByRole("tab", { name: "General" }).click();

  const showMessagesToggle = settingsPage.getByRole("switch", { name: "Show messages" });
  await expect(showMessagesToggle).toHaveAttribute("aria-checked", "true");

  await showMessagesToggle.click();

  expect(harness.calls).toContainEqual({
    command: "set_agent_message_visible",
    args: { visible: false },
  });
  await expect(showMessagesToggle).toHaveAttribute("aria-checked", "false");

  await showMessagesToggle.click();

  expect(
    harness.calls.filter((call) => call.command === "set_agent_message_visible"),
  ).toEqual([
    { command: "set_agent_message_visible", args: { visible: false } },
    { command: "set_agent_message_visible", args: { visible: true } },
  ]);
  await expect(showMessagesToggle).toHaveAttribute("aria-checked", "true");
});

test("pet visibility switch toggles the pet window", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: true,
    },
  });

  const settingsPage = await harness.openPage("settings");
  await settingsPage.getByRole("tab", { name: "General" }).click();

  const visibilityToggle = settingsPage.getByRole("switch", { name: "Show pet" });
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "true");

  await visibilityToggle.click();

  expect(harness.calls).toContainEqual({
    command: "toggle_pet_window_visibility",
    args: {},
  });
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "false");

  await visibilityToggle.click();

  expect(
    harness.calls.filter((call) => call.command === "toggle_pet_window_visibility"),
  ).toHaveLength(2);
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "true");
});

test("pet visibility switch follows system menu visibility changes", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: true,
    },
  });

  const settingsPage = await harness.openPage("settings");
  await settingsPage.getByRole("tab", { name: "General" }).click();

  const visibilityToggle = settingsPage.getByRole("switch", { name: "Show pet" });
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "true");

  await settingsPage.evaluate(() => {
    window.__copetTestEmit("copet-pet-window-visibility-changed", false);
  });
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "false");

  await settingsPage.evaluate(() => {
    window.__copetTestEmit("copet-pet-window-visibility-changed", true);
  });
  await expect(visibilityToggle).toHaveAttribute("aria-checked", "true");
});
