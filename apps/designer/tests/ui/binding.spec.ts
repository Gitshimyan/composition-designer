import { expect, test, type Page } from "@playwright/test";
import { installMock } from "./mockTauri";

async function openProject(page: Page) {
  await installMock(page, "valid");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await expect(page.getByText(/No diagnostics/)).toBeVisible();
}

async function selectInstance(page: Page, id: string) {
  await page.locator(".tree-node").filter({ hasText: id }).click();
  await expect(page.locator(".inspector")).toContainText(id);
}

function bindingRow(page: Page, text: string) {
  return page.locator(".bindings-block .binding-row").filter({ hasText: text });
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

test("moves a binding to another slot with Slot metadata", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "hello");

  await bindingRow(page, "app.content")
    .getByRole("button", { name: "Move…" })
    .click();

  const dialog = page.getByRole("dialog", { name: "Move hello" });
  await expect(dialog).toBeVisible();
  const aside = dialog.locator(".slot-choice").filter({ hasText: "aside" });
  await expect(aside).toContainText("optional_one");
  await expect(aside).toContainText("0 bound");
  await expect(aside).toContainText("accepts leaf.badge, leaf.widget");
  await aside.locator('input[type="radio"]').check();
  await dialog.getByRole("button", { name: "Review move" }).click();

  const review = page.getByRole("dialog", {
    name: /Move hello: app\.content .* app\.aside/,
  });
  await expect(review).toContainText("validated");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(page.locator(".bindings-block")).toContainText("app.aside");
  const commands = await lastCommands(page);
  expect(commands).toHaveLength(1);
  expect(commands[0]).toMatchObject({
    type: "update_binding",
    parent: "app",
    slot: "aside",
    child: "hello",
  });
  const snapshot = await page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: {
            snapshot: {
              model: {
                composition: {
                  bindings: Array<{
                    id: string;
                    parent: string;
                    slot: string;
                    child: string;
                  }>;
                };
              };
            };
          };
        }
      ).__COMPOSITION_TEST__.snapshot,
  );
  expect(
    snapshot.model.composition.bindings.find(
      (binding) => binding.id === "app-content",
    ),
  ).toMatchObject({ parent: "app", slot: "aside", child: "hello" });
});

test("detaching a binding shows the reachability diagnostic", async ({
  page,
}) => {
  await openProject(page);
  await selectInstance(page, "hello");

  await bindingRow(page, "app.content")
    .getByRole("button", { name: "Detach" })
    .click();

  const review = page.getByRole("dialog", {
    name: /Detach hello from app\.content/,
  });
  await expect(review).toContainText("rejected");
  await expect(review.locator(".review-diagnostics")).toContainText(
    "structure-unreachable-instance",
  );
  await expect(
    review.getByRole("button", { name: "Apply transaction" }),
  ).toBeDisabled();

  const stillBound = await page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: {
            snapshot: {
              model: {
                composition: {
                  bindings: Array<{ id: string; slot: string }>;
                };
              };
            };
          };
        }
      ).__COMPOSITION_TEST__.snapshot.model.composition.bindings.some(
        (binding) => binding.id === "app-content" && binding.slot === "content",
      ),
  );
  expect(stillBound).toBe(true);
});

test("creates a required-slot subtree in one transaction", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "app");

  await page
    .locator(".slot-row")
    .filter({ hasText: "content" })
    .getByRole("button", { name: "Add child…" })
    .click();

  const create = page.getByRole("dialog", { name: /app · content/ });
  await expect(create).toBeVisible();
  const rootFields = create.locator(
    "fieldset.draft-node.is-root > .dialog-fields",
  );
  await rootFields.getByLabel("Module").selectOption("ui.panel@1.0.0");

  const bodyGroup = create.getByRole("group", {
    name: "Slot body child",
  });
  await expect(bodyGroup).toBeVisible();
  await expect(bodyGroup).toContainText("required");
  const bodyFields = bodyGroup.locator(".dialog-fields");
  await bodyFields.getByLabel("Instance id").fill("card-body");
  await bodyFields.getByLabel("label").fill("Card body");
  await bodyFields.getByLabel("tone").selectOption("warn");
  await rootFields.getByLabel("Instance id").fill("card");
  await create.getByRole("button", { name: "Review create" }).click();

  const review = page.getByRole("dialog", {
    name: /Create card subtree \(2 instances\) under app\.content/,
  });
  await expect(review).toContainText("4 commands");
  await expect(review).toContainText("binding:bind-card");
  await expect(review).toContainText("binding:bind-card-body");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(
    page.locator(".tree-node").filter({ hasText: "card-body" }),
  ).toBeVisible();
  await expect(page.locator(".inspector")).toContainText("card");
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "create_instance",
    "create_instance",
    "create_binding",
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
                  instances: Array<{
                    id: string;
                    config: Record<string, unknown>;
                  }>;
                  bindings: Array<{
                    id: string;
                    parent: string;
                    slot: string;
                    child: string;
                  }>;
                };
              };
            };
          };
        }
      ).__COMPOSITION_TEST__.snapshot,
  );
  const child = snapshot.model.composition.instances.find(
    (instance) => instance.id === "card-body",
  );
  expect(child?.config).toMatchObject({ label: "Card body", tone: "warn" });
  const bindings = snapshot.model.composition.bindings;
  expect(
    bindings.find((binding) => binding.id === "bind-card"),
  ).toMatchObject({ parent: "app", slot: "content", child: "card" });
  expect(
    bindings.find((binding) => binding.id === "bind-card-body"),
  ).toMatchObject({ parent: "card", slot: "body", child: "card-body" });
});
