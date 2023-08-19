"%VULKAN_SDK%\Bin\glslangValidator.exe" -V shader.comp -o comp.spv
"%VULKAN_SDK%\Bin\glslangValidator.exe" -V shader.frag -o frag.spv
"%VULKAN_SDK%\Bin\glslangValidator.exe" -V shader.vert -o vert.spv
"%VULKAN_SDK%\Bin\glslangValidator.exe" -V texture_traverse.frag -o tex_frag.spv
"%VULKAN_SDK%\Bin\glslangValidator.exe" -V test.frag -o test.spv
pause