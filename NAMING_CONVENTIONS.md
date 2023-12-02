# Naming Conventions

## Chunks

*dimensions / dims (short):* 
Always represented as `Dimensions = (usize, usize, usize) (could be changed to bevy::math::UVec3)`, dims represent the physical 3d dimensions of a grid.
The format is the same as **Bevy** or **Minecraft** - (x, y, z), y - up, right-handed.
For example: a chunk could have dimensions of 16x64x16 ( = (16, 64, 16) ), the total blocks in the chunks would be 16 * 64 * 16 = 16,384

*width*:
The x dimension of the dimensions.
For example, a chunk with dimensions 10x20x30 would have a width of 10.

*length*:
The z dimension of the dimensions.
For example, a chunk with dimensions 10x20x30 would have a width of 30.

*height*:
The y dimension of the dimensions.
For example, a chunk with dimensions 10x20x30 would have a width of 20.

*grid:*:
The grid is the one dimensional chunk array, represented by `ChunkArr = [Block; width * length* height]`,
it contains each block in its corresponding index.
The block index is calculated as follows:
For a chunk with dimensions (width, height, length), and a block coordinates [x, y, z] (relative to chunk)
the block index is `y * (width * length) + z * width + x`

*chunk cords:*
The coordinate of the chunk in the world, represented by `ChunkCords = [i32; 2] (could be changed to bevy::math::Ivec2)`
For example: the point <0.0, 0.0, 0.0> in the world, is touching 4 chunks:
[0,0] in the direction of (+x, +z), [-1,0] in the direction of (-x,+z)
[0,-1] in the direction of (+x, -z), [-1,-1] in the direction of (-x,-z)

## Blocks

*index / block-index:*
the index of a block is its one-dimensional index in a non-compressed chunk array.
for example, a chunk with dimensions 16x64x16 

*position / block-pos:*
3d position in the chunk, represented by `[usize; 3], or UVec3`, calculated from the block index as followes:
    let h = (index / (length * width)) as usize;
    let l = ((index - h * (length * width)) / width) as usize;
    let w = (index - h * (length * width) - l * width) as usize;
    [w, h, l]

*face / side:*
the face of a cubic block
Top, Bottom, Right, Left, Back, Forward
+y , -y    , +x   , -x  , +z  , -z
0  , 1     , 2    , 3   , 4   , 5  (Into<usize>)

*neighbor:*
a block to the side of a different block, "side" refers to one of the following:

