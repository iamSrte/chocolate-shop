# Handmade Chocolate Shop

An online shop selling handmade chocolates, where customers build their own boxes or buy ready-made ones.

## Language

### Catalog

**Product**:
Anything a customer can put in a cart: a Preset Box, a Custom Box, or a Packaged Chocolate.
_Avoid_: Item, SKU, listing

**Preset Box**:
A curated, fixed set of Pieces in packaging, sold as a Product with its own page. Packs of Pieces (tubes, bags) are Preset Boxes too.
_Avoid_: Ready-made box, gift box, curated box, pack

**Custom Box**:
A box the customer fills piece by piece in the box builder.
_Avoid_: Build-a-box, bundle, variant

**Packaged Chocolate**:
A Product with its own recipe and packaging, not made of Pieces, such as a bar, bark or dragée.
_Avoid_: Standalone product, bar

### Pieces

**Chocolate**:
A kind of handmade piece defined by its Mold, together with the Shells, Fillings and Designs allowed with it. Never sold on its own.
_Avoid_: Piece type, flavor, bonbon, praline

**Mold**:
The form a Chocolate is cast in. It fixes the Chocolate's shape and size, and so which cavities it fits.
_Avoid_: Mould, shape

**Shell**:
The chocolate poured into the Mold to form the outside of a Piece, such as dark 70%, milk or ruby.
_Avoid_: Coating, couverture, base

**Filling**:
What is inside the Shell, such as ganache, praliné or caramel.
_Avoid_: Center, core, flavor

**Design**:
The decoration on the Shell's surface, such as paint, transfer sheet or dusting. Never changes shape.
_Avoid_: Decoration, finish, style

**Piece**:
One concrete Chocolate with a chosen Shell, Filling and Design. Boxes hold Pieces, and production makes Pieces.
_Avoid_: Variant, item, unit, bonbon

**Preset Piece**:
An admin-defined Shell, Filling and Design for a Chocolate, offered as a one-tap choice. Choosing it copies the combination into a Piece, so later edits do not change existing Pieces.
_Avoid_: Variant, signature piece, default

### Boxes

**Box Size**:
A packaging option with a fixed set of Cavities, used by both Custom Boxes and Preset Boxes.
_Avoid_: Box type, layout, packaging

**Cavity**:
One slot in a Box Size that accepts Chocolates of certain Molds.
_Avoid_: Slot, hole, position

### Pricing

**Ingredient**:
A raw material bought by the shop, with a price that changes over time.
_Avoid_: Material, supply

**Recipe**:
The Ingredient quantities and labor that make one Shell, Filling, Design, Chocolate base, Packaged Chocolate or Box Size packaging.
_Avoid_: Formula, BOM, bill of materials
