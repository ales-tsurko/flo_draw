#include <metal_stdlib>

#import "./bindings/metal_vertex2d.h"
#import "../simple/rasterizer.metal"

typedef struct {
    float4 v_Position [[position]];
    float v_TexCoord;
    float2 v_PaperCoord;
} GradientData;

vertex GradientData gradient_vertex(
      uint        vertex_id [[ vertex_id ]],
      constant    matrix_float4x4 *transform      [[ buffer(VertexInputIndexMatrix )]],
      constant    MetalVertex2D   *vertices       [[ buffer(VertexInputIndexVertices) ]]) {
    float4 position     = float4(vertices[vertex_id].pos[0], vertices[vertex_id].pos[1], 0.0, 1.0) * *transform;
    float2 tex_coord    = vertices[vertex_id].tex_coord;
    float2 paper_coord  = float2((position[0]+1.0)/2.0, 1.0-((position[1]+1.0)/2.0));

    GradientData data;

    data.v_Position     = position;
    data.v_TexCoord     = tex_coord[0];
    data.v_PaperCoord   = paper_coord;

    return data;
}
