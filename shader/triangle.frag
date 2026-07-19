#version 450

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec4 lightPos;
    vec4 camPos;
} ubo;

layout(set = 1, binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 fragPos;

layout(location = 0) out vec4 outColor;

void main() {
    float ambientStrength = 0.1;
    float specularStrength = 2;
    vec3 lightColor = vec3(1.0f, 1.0f, 1.0f);
    vec3 aNormal = normalize(fragNormal);
    vec3 camPos = vec3(-ubo.camPos.x, -ubo.camPos.y, -ubo.camPos.z);
    
    vec3 lightDir = normalize(ubo.lightPos.xyz - fragPos);
    vec3 viewDir = normalize(camPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, aNormal);

    float diff = max(dot(aNormal, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 8);
    vec3 specular = specularStrength * spec * lightColor;

    vec3 ambient = ambientStrength * lightColor;
    outColor = vec4((ambient + diffuse + specular) * texture(texSampler, fragTexCoord).rgb, 1.0);
}
