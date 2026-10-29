# Preset Pieces are copied on use, not linked

When a customer or admin picks a Preset Piece, its Shell, Filling and Design are copied into the Piece. Nothing keeps a link back to the Preset Piece. So when an admin edits a Preset Piece later, existing Preset Boxes, carts and orders keep the combination they were built with. An admin can still choose to apply the new combination to Preset Boxes as an explicit action.

## Considered Options

- **Live link**: editing a Preset Piece would update every Preset Box that uses it. Rejected because a Preset Box's contents, and so its price and product page, would change without anyone deciding to change them.
