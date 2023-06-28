use std::{ffi::CString, io::Cursor, mem};

use ash::{util::read_spv, vk, Device};

use crate::{interface::surface::SurfaceGroup, offset_of};

use super::{descriptor::DescriptorPool, image::ImageTarget};

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

#[derive(Clone)]
pub struct Shader {
    code: Vec<u32>,
    module: vk::ShaderModule,

    stage_info: vk::PipelineShaderStageCreateInfo,
}

#[derive(Clone)]
pub struct Pipe {
    pub image_target_list: Vec<ImageTarget>,

    pub descriptor_pool: DescriptorPool,

    pub shader_list: Vec<Shader>,

    pub pipe_layout: vk::PipelineLayout,
    pub pipe: vk::Pipeline,

    pub vertex_state: vk::PipelineVertexInputStateCreateInfo,
    pub vertex_assembly_stage: vk::PipelineInputAssemblyStateCreateInfo,

    pub viewport: Vec<vk::Viewport>,
    pub viewport_state: vk::PipelineViewportStateCreateInfo,

    pub raster_state: vk::PipelineRasterizationStateCreateInfo,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfo,
    pub blend_state: vk::PipelineColorBlendStateCreateInfo,
    pub dynamic_state: vk::PipelineDynamicStateCreateInfo,
}

// "../../shader/comp.spv"
// include_bytes!("../../shader/comp.spv")

impl Pipe {
    pub fn create_shader_module(
        &self,
        raw_code: &[u8; 0],
        shader_index: usize,
        device: &Device,
        stage_flag: vk::ShaderStageFlags,
    ) -> Self {
        unsafe {
            let mut result = self.clone();
            let mut shader = Shader::default();

            log::info!("Getting ShaderCode ...");
            let mut spv_file = Cursor::new(&raw_code[..]);

            shader.code = read_spv(&mut spv_file).expect("ERR_READ_SPV");
            let mod_info = vk::ShaderModuleCreateInfo::builder().code(&shader.code);

            shader.module = device
                .create_shader_module(&mod_info, None)
                .expect("ERR_VERTEX_MODULE");

            log::info!("Stage Creation ...");
            let enrty_name = CString::new("main").unwrap();
            shader.stage_info = vk::PipelineShaderStageCreateInfo {
                module: shader.module,
                p_name: enrty_name.as_ptr(),
                stage: stage_flag,
                ..Default::default()
            };

            result.shader_list[shader_index] = shader;

            result
        }
    }

    pub fn create_layout(&self, device: &Device) -> Self {
        unsafe {
            let mut result = self.clone();

            let info = vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&self.descriptor_pool.layout_list);

            log::info!("Creating PipelineLayout ...");
            result.pipe_layout = device.create_pipeline_layout(&info, None).unwrap();

            result
        }
    }

    pub fn create_vertex_stage(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            let vertex_binding_list = vec![vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];

            let vertex_attrib_list = vec![
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, uv) as u32,
                },
            ];

            result.vertex_state = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&vertex_attrib_list)
                .vertex_binding_descriptions(&vertex_binding_list)
                .build();

            result.vertex_assembly_stage = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };

            result
        }
    }

    pub fn create_viewport(&self, surface: &SurfaceGroup) -> Self {
        unsafe {
            let mut result = self.clone();

            result.viewport = vec![vk::Viewport {
                width: surface.render_res.width as f32,
                height: surface.render_res.height as f32,
                max_depth: 1.0,

                ..Default::default()
            }];

            let scissor = vec![surface.render_res.into()];
            result.viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&scissor)
                .viewports(&result.viewport)
                .build();

            result
        }
    }

    pub fn create_rasterization(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Rasterization ...");
            result.raster_state = vk::PipelineRasterizationStateCreateInfo {
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: vk::PolygonMode::FILL,
                ..Default::default()
            };

            result
        }
    }

    pub fn create_multisampling(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Multisample state ...");
            result.multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .build();

            result
        }
    }

    pub fn create_color_blending(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Blending ...");
            let blend_attachment_list = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,

                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,

                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,

                color_write_mask: vk::ColorComponentFlags::RGBA,
            }];

            result.blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&blend_attachment_list)
                .build();

            result
        }
    }

    pub fn create_dynamic_state(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Creating DynamicState ...");
            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            result.dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&dynamic_state)
                .build();

            result
        }
    }

    pub fn create_rendering(&self) -> Self {
        unsafe {
            let mut result = self.clone();

            let format_list = [interface.surface_format.format];
            let mut pipeline_rendering = vk::PipelineRenderingCreateInfoKHR::builder()
                .color_attachment_formats(&format_list);

            result
        }
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self {
            code: Default::default(),
            module: Default::default(),
            stage_info: Default::default(),
        }
    }
}
