use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::ptr::copy_nonoverlapping;
use ash::{vk, Device, Instance};
use nalgebra::{Vector2, Vector3};
use crate::renderer::buffer::{copy_buffer, create_buffer};
use crate::renderer::vertex::Vertex;

pub struct Model {
    pub index_count: u32,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
}

impl Model{
    pub fn new_from_path(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
        path: &str
    ) -> Result<Self, Box<dyn Error>>  {
        
        let (vertices, indices) = load_model(path)?;
        let (vertex_buffer, vertex_buffer_memory)= Self::create_vertex_buffer(
            instance,
            device,
            physical_device,
            &vertices,
            copy_queue,
            copy_command_pool,
        )?;
        let( index_buffer, index_buffer_memory )= Self::create_index_buffer(
            instance,
            device,
            physical_device,
            &indices,
            copy_queue,
            copy_command_pool,
        )?;
        let index_count = indices.len() as u32;

        Ok(Self{
            index_count, 
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        })
    }

    pub fn new_from_raw_data(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
        vertices : Vec<Vertex>,
        indices: Vec<u32>
    ) -> Result<Self, Box<dyn Error>>  {
        let (vertex_buffer, vertex_buffer_memory)= Self::create_vertex_buffer(
            instance,
            device,
            physical_device,
            &vertices,
            copy_queue,
            copy_command_pool,
        )?;
        let( index_buffer, index_buffer_memory )= Self::create_index_buffer(
            instance,
            device,
            physical_device,
            &indices,
            copy_queue,
            copy_command_pool,
        )?;
        let index_count = indices.len() as u32;

        Ok(Self{
            index_count,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        })
    }
    
    fn create_vertex_buffer(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        vertices: &[Vertex],
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), Box<dyn Error>> {
        let size = (size_of::<Vertex>() * (vertices.len())) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = unsafe {
            device.map_memory(
                staging_buffer_memory,
                0,
                size,
                vk::MemoryMapFlags::empty(),
            )
        }?;
        unsafe { copy_nonoverlapping(vertices.as_ptr(), memory.cast(), vertices.len()); }

        unsafe { device.unmap_memory(staging_buffer_memory); }

        let (buffer, buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(device, copy_queue, copy_command_pool, staging_buffer, buffer, size)?;

        unsafe { device.destroy_buffer(staging_buffer, None); }
        unsafe { device.free_memory(staging_buffer_memory, None); }

        Ok((
            buffer,
            buffer_memory,
        ))
    }
    
    fn create_index_buffer(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        indices: &[u32],
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
    )-> Result<(vk::Buffer, vk::DeviceMemory), Box<dyn Error>> {
        let size = (size_of::<u32>() * indices.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        // Copy (staging)

        let memory = unsafe { device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty()) }?;

        unsafe { copy_nonoverlapping(indices.as_ptr(), memory.cast(), indices.len()); }

        unsafe { device.unmap_memory(staging_buffer_memory); }

        // Create (index)

        let (buffer, buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;



        // Copy (index)

        copy_buffer(device, copy_queue, copy_command_pool, staging_buffer, buffer, size)?;

        // Cleanup

        unsafe { device.destroy_buffer(staging_buffer, None); }
        unsafe { device.free_memory(staging_buffer_memory, None); }

        Ok((buffer, buffer_memory))
    }
    
}

fn load_model(path: &str) -> anyhow::Result<(Vec<Vertex>, Vec<u32>)> {
    let mut reader = BufReader::new(File::open(path)?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { triangulate: true, ..Default::default() },
        |_| Ok(Default::default()),
    )?;

    let mut vertices = vec![];
    let mut indices = vec![];
    //let mut unique_vertices = HashMap::new();
    //let correction = nalgebra::Matrix4::from_angle_x(Rad(std::f32::consts::FRAC_PI_2));
    for model in &models {

        for i in 0..model.mesh.indices.len() {
            let pi = model.mesh.indices[i] as usize;
            let ti = model.mesh.texcoord_indices[i] as usize;
            let ni = model.mesh.normal_indices[i] as usize;

            let vertex = Vertex {
                // pos:  correction.transform_vector(
                //     model.mesh.positions[3 * pi],
                //     model.mesh.positions[3 * pi + 1],
                //     model.mesh.positions[3 * pi + 2],
                // ) ),
                // normal: correction.transform_vector(vec3(
                //     model.mesh.positions[3 * ni],
                //     model.mesh.positions[3 * ni + 1],
                //     model.mesh.positions[3 * ni + 2],
                // )),
                // color: vec3(1.0, 1.0, 1.0),
                // tex_coord: vec2(
                //     model.mesh.texcoords[2 * ti],
                //     1.0 - model.mesh.texcoords[2 * ti + 1],
                // ),


                pos:  Vector3::new (
                    model.mesh.positions[3 * pi],
                    model.mesh.positions[3 * pi + 1],
                    model.mesh.positions[3 * pi + 2],
                ),
                
                normal: Vector3::new (
                    model.mesh.positions[3 * ni],
                    model.mesh.positions[3 * ni + 1],
                    model.mesh.positions[3 * ni + 2],
                ),
                color: Vector3::new (1.0, 1.0, 1.0),
                tex_coord: Vector2::new(
                    model.mesh.texcoords[2 * ti],
                    1.0 - model.mesh.texcoords[2 * ti + 1],
                ),
            };

            vertices.push(vertex);
            indices.push(i as u32);
        }
    };



    Ok((vertices, indices))
}