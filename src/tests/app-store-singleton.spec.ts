import { expect, test } from "@playwright/test";

import { createAppHarness, hoverpet } from "./app-harness";

test("PetWindow bootstrap issues exactly one logical fetch (no dual-instance)", async ({
  browser,
}) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: hoverpet.id,
      pets: [hoverpet],
      onboardingComplete: false,
    },
  });

  const page = await harness.openPage("pet");
  await expect(page.getByRole("img", { name: "HoverPet" })).toBeVisible();

  // React Strict Mode double-mounts the bootstrap effect, so each invoke runs
  // exactly twice. The dual-instance bug (PetWindow + useLayeredPetState both
  // calling useAppData) pushed these counts to 4. A drift in either direction
  // indicates a regression worth investigating.
  expect(harness.invocations("get_app_state")).toHaveLength(2);
  expect(harness.invocations("get_runtime_status")).toHaveLength(2);
  expect(harness.invocations("list_agent_adapters")).toHaveLength(2);
  expect(harness.invocations("list_codex_pets")).toHaveLength(2);
  expect(harness.invocations("get_pet_window_visible")).toHaveLength(2);
});

test("dismissed agent message stays dismissed across pet-state events", async ({
  browser,
}) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: hoverpet.id,
      pets: [hoverpet],
      onboardingComplete: false,
    },
  });

  const page = await harness.openPage("pet");
  await expect(page.getByRole("img", { name: "HoverPet" })).toBeVisible();

  const agentMessage = {
    agent: "codex",
    displayName: "Codex",
    text: "thinking about it",
    updatedAtMs: 1_000,
  };

  await harness.emitRuntimeUpdate(page, {
    currentState: { state: "jumping" },
    messages: [agentMessage],
  });

  const bubble = page.getByTestId("pet-agent-message");
  await expect(bubble).toBeVisible();

  await bubble.getByRole("button", { name: "Dismiss" }).click();
  await expect(bubble).toHaveCount(0);

  await harness.emitRuntimeUpdate(page, {
    currentState: { state: "jumping" },
    messages: [agentMessage],
  });

  await expect(page.getByTestId("pet-agent-message")).toHaveCount(0);
});
