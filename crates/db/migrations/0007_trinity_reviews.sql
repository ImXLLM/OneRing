-- 0007_trinity_reviews — the Mairon seal on marketplace listings + citizens' 5-star votes
-- (Wave 3 — Sovereign Trinity Studio Pipeline)
--
-- Rules:
--  * new columns on "Product" carry safe defaults so legacy databases and the
--    legacy migration path upgrade without failure (same convention as 0005).
--  * the "ProductReview" table has a UNIQUE (product, voter) one-vote lock —
--    a shill cannot manufacture ratings; database backing for the domain rule.
--  * the Mairon score and badge land on "Product" only through the two-key gate
--    (Mairon verdict + builder confirmation).

-- the Mairon arbitration seal on the product (two-key publication gate)
ALTER TABLE "Product" ADD COLUMN "maironScore" INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "Product" ADD COLUMN "maironBadge" TEXT NOT NULL DEFAULT '';

-- real citizen votes — stars 1..=5, comment, and verified-purchase weight
CREATE TABLE IF NOT EXISTS "ProductReview" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "productId" TEXT NOT NULL,
    "reviewerId" TEXT NOT NULL,
    "stars" INTEGER NOT NULL,
    "comment" TEXT NOT NULL,
    "verifiedPurchase" INTEGER NOT NULL DEFAULT 0,
    "maironScoreAtReview" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS "ProductReview_product_reviewer_uniq"
    ON "ProductReview"("productId", "reviewerId");

CREATE INDEX IF NOT EXISTS "ProductReview_productId_idx"
    ON "ProductReview"("productId");
