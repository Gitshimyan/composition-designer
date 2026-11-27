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

async function showView(page: Page, name: "Structure" | "Connections" | "Dependencies") {
  await page.getByRole("tab", { name: new RegExp(name) }).click();
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

async function composition(page: Page) {
  return page.evaluate(
    () =>
      (
        window as unknown as {
          __COMPOSITION_TEST__: {
            snapshot: {
              model: {
                composition: {
                  connections: Array<{
                    id: string;
                    from: { instance: string; port: string };
                    to: { instance: string; port: string };
                  }>;
                };
              };
            };
          };
        }
      ).__COMPOSITION_TEST__.snapshot.model.composition,
  );
}

test("switches between relation views without losing selection", async ({
  page,
}) => {
  await openProject(page);
  await selectInstance(page, "box");

  await showView(page, "Connections");
  await expect(page.locator(".port-card.is-selected")).toContainText("box");
  await expect(page.locator(".connection-row.is-selected")).toContainText(
    "box-log-line",
  );

  await showView(page, "Dependencies");
  await expect(page.locator(".dependency-group.is-selected")).toContainText(
    "box",
  );

  await showView(page, "Structure");
  await expect(page.locator(".inspector")).toContainText("box");
});

test("updates a port connection through review", async ({ page }) => {
  await openProject(page);
  await showView(page, "Connections");

  await page
    .locator(".connection-row")
    .filter({ hasText: "box-log-line" })
    .getByRole("button", { name: "Edit" })
    .click();

  const dialog = page.getByRole("dialog", { name: "Edit connection" });
  await expect(dialog.getByLabel("From instance")).toHaveValue("box");
  await expect(dialog.getByLabel("From port")).toHaveValue("log");
  await expect(dialog.getByLabel("To instance")).toHaveValue("logger");
  await expect(dialog.getByLabel("To port")).toHaveValue("line");
  await dialog.getByLabel("To port").selectOption("audit");
  await dialog.getByRole("button", { name: "Review connection" }).click();

  const review = page.getByRole("dialog", {
    name: "Update connection box-log-line",
  });
  await expect(review).toContainText("validated");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  const replaced = (await composition(page)).connections.find(
    (connection) => connection.id === "box-log-line",
  );
  expect(replaced?.to).toEqual({ instance: "logger", port: "audit" });
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "update_connection",
  ]);
});

test("creates a port connection from the connection view", async ({ page }) => {
  await openProject(page);
  await showView(page, "Connections");

  await page.getByRole("button", { name: "New connection" }).click();
  const dialog = page.getByRole("dialog", { name: "New connection" });
  await expect(dialog.getByLabel("Connection id")).toHaveValue("connect-1");
  await expect(dialog.getByLabel("From instance")).toHaveValue("box");
  await expect(dialog.getByLabel("From port")).toHaveValue("log");
  await dialog.getByLabel("To port").selectOption("audit");
  await dialog.getByRole("button", { name: "Review connection" }).click();

  const review = page.getByRole("dialog", {
    name: "Create connection connect-1",
  });
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  const created = (await composition(page)).connections.find(
    (connection) => connection.id === "connect-1",
  );
  expect(created?.from).toEqual({ instance: "box", port: "log" });
  expect(created?.to).toEqual({ instance: "logger", port: "audit" });
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "create_connection",
  ]);
});

test("deletes a port connection after review", async ({ page }) => {
  await openProject(page);
  await showView(page, "Connections");

  await page
    .locator(".connection-row")
    .filter({ hasText: "box-log-line" })
    .getByRole("button", { name: "Delete" })
    .click();

  const review = page.getByRole("dialog", {
    name: "Delete connection box-log-line",
  });
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  expect((await composition(page)).connections).toHaveLength(0);
  const commands = await lastCommands(page);
  expect(commands.map((command) => command.type)).toEqual([
    "delete_connection",
  ]);
});

test("projects dependency requirements and providers read-only", async ({
  page,
}) => {
  await openProject(page);
  await showView(page, "Dependencies");

  await expect(page.locator(".relation-readonly")).toHaveText("read-only");
  const group = page.locator(".dependency-group").filter({ hasText: "box" });
  await expect(group).toContainText("logger");
  await expect(group).toContainText("svc.logger@1.0.0");
  await expect(group).toContainText("required");
  await expect(group).toContainText("resolved");
  await expect(group).toContainText("provider:");
  await expect(group).toContainText("explicit");
  await expect(group).toContainText("candidates: logger");
  await expect(page.locator(".relation-note")).toContainText(
    "Provider mapping cannot be edited",
  );
});
