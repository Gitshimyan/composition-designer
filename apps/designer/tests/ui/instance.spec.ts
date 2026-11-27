import { expect, test, type Page } from "@playwright/test";
import { installMock, type MockMode } from "./mockTauri";

async function openProject(page: Page, mode: MockMode = "valid") {
  await installMock(page, mode);
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await expect(page.getByText(/No diagnostics/)).toBeVisible();
}

async function selectInstance(page: Page, id: string) {
  await page.locator(".tree-node").filter({ hasText: id }).click();
  await expect(page.locator(".inspector")).toContainText(id);
}

function actions(page: Page) {
  return page.locator(".instance-actions");
}

async function lastCommands(page: Page) {
  return page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: {
            lastCommands: Array<{ type: string; [key: string]: unknown }>;
          };
        }
      ).__COMPOSITION_TEST__.lastCommands,
  );
}

test("renames an instance through review", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "hello");

  await actions(page).getByRole("button", { name: "Rename" }).click();
  await page.getByLabel("New instance id").fill("greeting");
  await actions(page).getByRole("button", { name: "Review", exact: true }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Rename hello → greeting");
  await expect(review).toContainText("validated");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(
    page.locator(".tree-node").filter({ hasText: "greeting" }),
  ).toBeVisible();
  await expect(page.locator(".inspector")).toContainText("greeting");
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual(["rename_instance"]);
  expect(commands[0].to).toBe("greeting");
});

test("changes the module and fills newly required config", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "hello");

  await page.getByLabel("Instance module").selectOption("leaf.badge@1.0.0");
  await actions(page).getByLabel("tone").selectOption("warn");
  await actions(page)
    .getByRole("button", { name: "Review module change" })
    .click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Change hello module to leaf.badge@1.0.0");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(page.locator(".inspector")).toContainText("leaf.badge");
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "update_instance_module",
    "set_config",
  ]);
  expect(commands[1]).toMatchObject({
    instance: "hello",
    property: "tone",
    value: "warn",
  });
});

test("creates a child instance and its first binding in one transaction", async ({
  page,
}) => {
  await openProject(page);
  await selectInstance(page, "app");

  await page
    .locator(".slot-row")
    .filter({ hasText: "content" })
    .getByRole("button", { name: "Add child…" })
    .click();

  const create = page.getByRole("dialog", { name: /app · content/ });
  await expect(create).toBeVisible();
  await create.getByLabel("Instance id").fill("footer");
  await create.getByLabel("label").fill("Footer");
  await create.getByRole("button", { name: "Review create" }).click();

  const review = page.getByRole("dialog", {
    name: "Create footer under app.content",
  });
  await expect(review).toContainText("2 commands");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(
    page.locator(".tree-node").filter({ hasText: "footer" }),
  ).toBeVisible();
  await expect(page.locator(".inspector")).toContainText("footer");
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "create_instance",
    "create_binding",
  ]);
  const snapshot = await page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: {
            snapshot: {
              model: {
                composition: {
                  instances: Array<{ id: string; config: Record<string, unknown> }>;
                  bindings: Array<{ parent: string; slot: string; child: string }>;
                };
              };
            };
          };
        }
      ).__COMPOSITION_TEST__.snapshot,
  );
  const created = snapshot.model.composition.instances.find(
    (instance) => instance.id === "footer",
  );
  expect(created?.config.label).toBe("Footer");
  expect(created?.config.tone).toBe("info");
  expect(
    snapshot.model.composition.bindings.some(
      (binding) =>
        binding.parent === "app" &&
        binding.slot === "content" &&
        binding.child === "footer",
    ),
  ).toBe(true);
});

test("deletes an instance subtree with its relations after impact review", async ({
  page,
}) => {
  await openProject(page);
  await selectInstance(page, "box");

  await actions(page).getByRole("button", { name: "Delete" }).click();
  const impact = page.locator(".impact-panel");
  await expect(impact).toContainText("box");
  await expect(impact).toContainText("hint");
  await expect(impact).toContainText("app-content-box");
  await expect(impact).toContainText("box-body");
  await impact.getByRole("button", { name: "Review delete" }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Delete box and 1 descendant(s)");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(
    page.locator(".tree-node").filter({ hasText: "box" }),
  ).toHaveCount(0);
  await expect(
    page.locator(".tree-node").filter({ hasText: "hint" }),
  ).toHaveCount(0);
  await expect(page.locator(".inspector")).toContainText("app");
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "delete_instance",
    "delete_instance",
  ]);
  expect(commands.map((command) => command.id)).toEqual(["hint", "box"]);
});

test("disables deleting the root instance", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "app");
  await expect(
    actions(page).getByRole("button", { name: "Delete" }),
  ).toBeDisabled();
});

test("blocks an invalid rename locally before prepare", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "hello");

  await actions(page).getByRole("button", { name: "Rename" }).click();
  await page.getByLabel("New instance id").fill("box");
  await actions(page).getByRole("button", { name: "Review", exact: true }).click();

  await expect(actions(page).locator(".config-error")).toContainText(
    "already exists",
  );
  const prepareCalls = await page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: { prepareCalls: number };
        }
      ).__COMPOSITION_TEST__.prepareCalls,
  );
  expect(prepareCalls).toBe(0);
});

test("shows a rejected instance transaction without applying it", async ({
  page,
}) => {
  await openProject(page, "edit-failure");
  await selectInstance(page, "hello");

  await actions(page).getByRole("button", { name: "Rename" }).click();
  await page.getByLabel("New instance id").fill("greeting");
  await actions(page).getByRole("button", { name: "Review", exact: true }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("rejected");
  await expect(review.locator(".review-diagnostics")).toContainText(
    "contract-property-type-mismatch",
  );
  await expect(
    review.getByRole("button", { name: "Apply transaction" }),
  ).toBeDisabled();
  await expect(
    page.locator(".tree-node").filter({ hasText: "hello" }),
  ).toBeVisible();
});
