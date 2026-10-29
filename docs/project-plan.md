# Handmade Chocolate Shop — Project Plan

## Overview

An online shop for handmade chocolates where buying is part of the fun: customers pick a box size and watch it fill as they add pieces. Shoppers who don't want to build can buy ready-made preset boxes or other standalone packaged chocolates in one click.

Goals:

- Make building a box feel playful and visual, not like filling a form.
- Keep prices accurate while ingredient costs change constantly, by computing them from recipes.
- Reach new customers through search, with fast pre-rendered product and gift pages.
- Ship a small v1 quickly, then grow features based on real orders.

## Core features

### Box builder

The customer picks a box size, then fills it from a 2D or 3D view of the real box. Each piece drops into the next empty cavity with a short animation, and the lid closes when the user is done.

- Mobile: tap a chocolate to add it to the next empty slot. Desktop can add drag-and-drop later.
- Rules live as data: cavity shapes (round, square, bar), which chocolates fit which cavity, max and min pieces per flavor, box must be full to check out.
- Filters for dietary needs (vegan, nut-free) and a live "X of 16 filled" counter with running price.
- The server re-validates every box at checkout; the client check is only for instant feedback.

### Preset boxes

Curated boxes sold as normal products, each with its own page. They are the main SEO pages and the fastest way to buy.

### Ingredient-based pricing

Each chocolate has a recipe: grams of each ingredient, plus labor, packaging and overhead per piece. When an admin updates ingredient prices, all chocolate and box prices are recomputed.

- Price = cost × margin, rounded to clean numbers. Box price = packaging + sum of pieces.
- Ingredient price history is kept.
- A price jump above a threshold (e.g. 15%) waits for admin approval.
- Cart prices are locked for about 30 minutes, and every order stores a snapshot of the prices it was sold at.
- All money is stored as integers (tomans), never floats.

### Orders

States: paid → in production → packed → shipped → delivered. Because everything is handmade, production capacity per day is limited and customers choose a delivery date. Heat-sensitive shipping needs zone limits or cold packaging in summer.

### SEO

- Server-rendered or pre-rendered pages for chocolates, preset boxes and blog posts. The builder itself is client-side and won't rank.
- Product structured data (schema.org) with price, kept in sync with dynamic pricing; sitemap; clean Persian slugs.
- Occasion content: Yalda, Nowruz, Valentine's, corporate gifts.
- Product feed for price-comparison sites like Torob, and a Google Business Profile for local search.

## Scope: v1 and later

v1 sells real boxes end to end; everything else waits for real order data.

| Area | v1 | Later |
| --- | --- | --- |
| Catalog | Chocolates + 3–4 preset boxes, pre-rendered pages | Seasonal limited flavors with countdown |
| Pricing | Ingredients + recipes tables, recompute button | Approval workflow, alerts on big jumps |
| Checkout | Phone OTP login, address, payment gateway, gift message | Scheduled delivery calendar with daily capacity |
| Admin | Order list + status changes, ingredient price editing | Production planning, reports |
| Growth | Basic SEO, sitemap, schema | Blog, Torob feed, taste quiz, subscriptions, reviews, corporate orders, loyalty/referral |

## Tech stack and architecture

TanStack Start renders pages for SEO and only calls the API; a Rust Axum service owns all business logic.

&#91;embedded content: system architecture · 6 parts\]

- **Frontend:** TanStack Start with SSR and prerendering for catalog, box and blog pages. The builder runs client-side.
- **Backend:** Axum + SQLx + PostgreSQL.
- **Shared types:** `utoipa` generates an OpenAPI spec; the TypeScript client is generated from it, so the two codebases stay in sync.
- **Pricing and box rules:** pure Rust modules with unit tests. Prices recompute when an admin saves ingredient prices; no job queue needed yet.
- **Auth:** phone OTP via an SMS provider.
- **Payments:** a local gateway such as Zarinpal.
- **Hosting:** a domestic host such as Liara or ArvanCloud, with object storage for images.
- **Admin:** routes inside the same TanStack Start app, behind an admin role.

### Admin

- Orders: list, filters by status and delivery date, status changes.
- Ingredients: edit prices, see history.
- Pricing: preview recomputed prices, approve big jumps.
- Catalog: chocolates, recipes, box sizes, cavities, rules, preset boxes.
- Capacity: pieces per day (later).

## Inspiration

| Site | What to look at |
| --- | --- |
| [Cocoa Dolce – Build a Box](https://shop.cocoadolce.com/pages/build-your-own-chocolate-box) | Size first, then fill; FAQ under the builder for SEO |
| [Dallmann Confections](https://dallmannconfections.com/blogs/chocolate/build-dietary-friendly-chocolate-box) | Click-to-fill boxes from 5 to 63 pieces, dietary filters, SEO blog posts |
| [Rococo Chocolates](https://rococochocolates.com/products/build-your-custom-selection-box) | Shopify bundle approach: each customer box becomes a variant (what to avoid) |
| [CocoaBella (Trend Hunter)](https://www.trendhunter.com/trends/build-your-own-custom-box-of-chocolates) | Early drag-and-drop virtual gift box |
| [Pacdora chocolate box mockups](https://www.pacdora.com/tools/chocolate-box-mockup) | Editable box mockups for prototyping box art |
