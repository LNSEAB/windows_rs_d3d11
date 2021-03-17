struct VSInput {
    float3 position: POSITION;
    float4 color: COLOR;
};

struct VSOutput {
    float4 sv_position: SV_POSITION;
    float4 color: COLOR;
};

VSOutput vs_main(VSInput input) {
    VSOutput output;
    output.sv_position = float4(input.position, 1.0f);
    output.color = input.color;
    return output;
}

float4 ps_main(VSOutput vs): SV_TARGET {
    return vs.color;
}