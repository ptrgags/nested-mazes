# Nested Mazes

## 2022-05-20 Initial DFS Maze Generation

So far I have a basic grid and a `DFSMaze` class to connect cells together
into a maze.

Next steps:

* Compute the solution path
* Generate a glTF with the results as a feature ID texture 
  (via `EXT_mesh_features`)

## 2022-05-23 Productive Weekend

This weekend and this morning I got a few more parts
of this done:

* Generated a PNG image for the feature ID texture. I manually made a glTF for 
  testing
* Tried making a shader to visualize the results. It's working, though I'm 
  getting seam artifacts. I need to learn more about texture atlasing
* Subdivided a parent tile into 4 children and propagated walls and boundary
  conditions. not 100% tested, but I need tileset output to see for sure. I
  also need to make a method to recursively subdivide until a given level.
* Started working on glTF output. I have a function to generate the geometry
  buffer, the next step is generating the GLB files. I started working on the
  JSON part but still have to compute the buffer offsets and such.

Next steps:

* Generate GLB files
* Generate an implicit tileset JSON
* Generate solution path
* Make a viewer in CesiumJS

# 2022-05-25 Getting Closer

Yesterday and today, I've been chipping away at the tileset
generation. Now I have a `Tileset` class which copies static
files to the output directory and does a DFS to generate
the `Tiles`. The JSON for the GLB file is nearly complete,
but then I still need to generate the GLB binary file.
Shouldn't be too long before I have something I can render
in CesiumJS.

Next Steps:

* Figure out correct matrix translations for GLB files
* Finish generating GLB files and tileset
* Make a CesiumJS viewer
* Figure out how to generate the solution paths