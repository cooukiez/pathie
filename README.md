## Disclaimer: This is just me learning computer graphics.
# Pathie
This is some kind of tracing project with voxels, I'm not sure what the current status is and I can't guarantee it'll run.
# Implemented
* vulkan interface
* render pipeline
  * pretty sure, graphics is in use, compute pipeline also exists
  * older releases have a compute pipeline and some also graphics, I couldn't decide whether to use
* basic octree data structure, bitwise storage, properties of voxels are store in u32's
* so the octree is just a least of u32's which is more mem efficient and better to align
* a lot of optimizations
* octree to texture, I'm not so sure if it is a 2d or 3d
* sdf generation with jump flooding
* different tracing algorithms for different structures
* octree traversal from abje (i think it was his name) from shadertoy
