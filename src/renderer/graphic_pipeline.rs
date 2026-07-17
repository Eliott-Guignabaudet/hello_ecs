use std::error::Error;
use ash::{vk, Device};
use ash::vk::ColorComponentFlags;

pub struct GraphicsPipeline{
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set_layout_material: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    
}


impl GraphicsPipeline {
    pub fn new(
        device: &Device,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        vertex_binding_description: vk::VertexInputBindingDescription,
        vertex_attribute_descriptions: [vk::VertexInputAttributeDescription; 4],
        msaa_samples: vk::SampleCountFlags,
        swapchain_image_count: u32,
    ) -> Result<Self, Box<dyn Error>> {
        
        let descriptor_set_layout = Self::create_descriptor_set_layout(device)?;
        let descriptor_set_layout_material = Self::create_descriptor_set_layout_material(device)?;
        
        
        let (pipeline, pipeline_layout) = Self::create_pipeline(
            device,
            swapchain_extent,
            render_pass,
            &[descriptor_set_layout, descriptor_set_layout_material],
            vertex_binding_description,
            vertex_attribute_descriptions,
            msaa_samples,
        )?;
        
        let descriptor_pool = Self::create_descriptor_pool(
            device,
            swapchain_image_count * 2
        )?;
        
        Ok(Self{
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            descriptor_set_layout_material,
            descriptor_pool,
        })
        
    }
    
    fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout, Box<dyn Error>> {
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        
        let bindings = &[ubo_binding];
        let info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(bindings);

        let handle = unsafe { device.create_descriptor_set_layout(&info, None) }?;
        Ok( handle )
    }

    fn create_descriptor_set_layout_material(device: &Device) -> Result<vk::DescriptorSetLayout, Box<dyn Error>> {
        
        let material_ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let sampler_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = &[sampler_binding, material_ubo_binding];
        let info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(bindings);

        let handle = unsafe { device.create_descriptor_set_layout(&info, None) }?;
        Ok( handle )
    }
    
    fn create_pipeline(
        device: &Device,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        set_layouts: &[vk::DescriptorSetLayout],
        vertex_binding_description: vk::VertexInputBindingDescription,
        vertex_attribute_descriptions: [vk::VertexInputAttributeDescription; 4],
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout), Box<dyn Error>> {
        let vert = include_bytes!("../../shader/vert.spv");
        let frag = include_bytes!("../../shader/frag.spv");

        let vert_shader_module = unsafe { Self::create_shader_module(device, vert)?};
        let frag_shader_module = unsafe { Self::create_shader_module(device, frag)?};

        let vert_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(c"main");

        let frag_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(c"main");

        let vertex_binding_descriptions = &[vertex_binding_description];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);


        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent);

        let viewports = &[viewport];
        let scissors = &[scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(viewports)
            .scissors(scissors);

        // Rasterization State

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        // Multisample State

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .rasterization_samples(msaa_samples);
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // Color Blend State
        
        let attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(ColorComponentFlags::RGBA,)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let vert_push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64 /* 16 × 4 byte floats */);

        let frag_push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(64)
            .size(4);

        let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];
        // Layout
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(push_constant_ranges);

        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;


        let stages = &[vert_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::default()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
        }.unwrap()[0];

        unsafe { device.destroy_shader_module(frag_shader_module, None); }
        unsafe { device.destroy_shader_module(vert_shader_module, None); }

        Ok((pipeline, pipeline_layout))
    }

    unsafe fn create_shader_module(
        device: &Device,
        bytecode: &[u8],
    ) -> anyhow::Result<vk::ShaderModule> {
        assert!(bytecode.len() % 4 == 0, "SPIR-V byte length must be a multiple of 4");
        let code: Vec<u32> = bytecode.chunks_exact(4)
            .map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        let info = vk::ShaderModuleCreateInfo {
            code_size: bytecode.len(),
            p_code:    code.as_ptr(),
            ..Default::default()
        };
        Ok(device.create_shader_module(&info, None)?)
    }
    
    fn create_descriptor_pool(
        device: &Device, 
        descriptor_count: u32
    ) -> Result<vk::DescriptorPool, Box<dyn Error>> {
        let ubo_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(descriptor_count);
        let sampler_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(descriptor_count);


        let pool_sizes = &[ubo_size, sampler_size];
        let info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(pool_sizes)
            .max_sets(descriptor_count);
        let handle = unsafe { device.create_descriptor_pool(&info, None) }?;
        Ok(handle )
    }
}