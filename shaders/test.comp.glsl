#version 460

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0, rgba8) uniform image2D image;

void main() {
  vec2 norm_coords = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(image));
  imageStore(image, ivec2(gl_GlobalInvocationID.xy), vec4(norm_coords, 0.0, 1.0));
}

