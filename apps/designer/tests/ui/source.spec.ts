import { expect, test, type Locator, type Page } from "@playwright/test";
import { sourceText } from "./fixtures";
import { installMock } from "./mockTauri";

async function openSource(page: Page) {
  await installMock(page, "valid");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await page.getByRole("tab", { name: "TOML Source" }).click();
  const editor = page.locator(".cm-content");
  await expect(editor).toContainText("Minimal Project");
  return editor;
}

async function replaceSource(page: Page, editor: Locator, text: string) {
  await editor.click();
  await page.keyboard.press("Control+A");
  await page.keyboard.insertText(text);
}

test("edits source, reviews text diff and applies", async ({ page }) => {
  const editor = await openSource(page);
  const after = sourceText.replace(
    'title = "Minimal Project"',
    'title = "Edited from source"',
  );
  await replaceSource(page, editor, after);
  await page.getByRole("button", { name: "Review source" }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Edit composition source");
  await expect(review).toContainText("Source change");
  await expect(review).toContainText("Edited from source");
  await review.getByRole("button", { name: "Apply transaction" }).click();

  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(editor).toContainText("Edited from source");
});

test("rejects invalid source and keeps the draft until discard", async ({ page }) => {
  const editor = await openSource(page);
  const invalid = sourceText.replace(
    'title = "Minimal Project"',
    "title = 42",
  );
  await replaceSource(page, editor, invalid);
  await page.getByRole("button", { name: "Review source" }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("rejected");
  await expect(review.locator(".review-diagnostics")).toContainText(
    "contract-property-type-mismatch",
  );
  await expect(
    review.getByRole("button", { name: "Apply transaction" }),
  ).toBeDisabled();
  await review.getByRole("button", { name: "Cancel" }).click();
  await expect(editor).toContainText("title = 42");

  await page.getByRole("button", { name: "Discard" }).click();
  await expect(editor).toContainText('title = "Minimal Project"');
});
