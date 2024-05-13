# Introduction

Zero sized components have the unique advantage that they don't need to be stored anywhere. They can effectively exist on the archetype level. What do I mean by this? First they don't need to be stored anywhere. They store no information and references to them can be created from thin air. Secondly there's no modifying them. As stated they contain no data, so there's no data to modify. All information on needs to find and construct a ZST component for a given entity can be read from the archetype information for said entity.

We don't take advantage of this fact at all. We still construct a `Column` or a `ComponentSparseSet` for every ZST component. `ComponentSparseSet` egregiously wastes space holding data about all the entities holding our ZST component.

## Solution

Introduce a new `Archetypal` storage type. This storage type is imaginary. It doesn't actually exist. Only ZST types can be stored in this storage. All this means that reading data from the `Archetypal` storage is as simple as:

1. Constructing a correctly aligned pointer to your zero sized component.
2. Dereferencing said pointer.

Adding a component using the `Archetypal` storage to an entity should be as fast if not faster than it is for `SparSet` components