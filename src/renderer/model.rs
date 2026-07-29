use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::ptr::copy_nonoverlapping;
use std::sync::Arc;
use ash::{vk, Device, Instance};
use nalgebra::{min, Vector2, Vector3};
use crate::renderer::buffer::{copy_buffer, create_buffer};
use crate::renderer::vertex::Vertex;

pub struct Model {
    pub index_count: u32,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub min_pos: Vector3<f32>,
    pub max_pos: Vector3<f32>,
    
    device: Arc<Device>
}

impl Model{
    pub fn new_from_path(
        instance: &Instance,
        device: Arc<Device>,
        physical_device: vk::PhysicalDevice,
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
        path: &str,
        correction: nalgebra::Matrix4<f32>
    ) -> Result<Self, Box<dyn Error>>  {
        
        let (vertices, indices, min_pos, max_pos) = load_model(path, correction)?;
        let (vertex_buffer, vertex_buffer_memory)= Self::create_vertex_buffer(
            instance,
            &device,
            physical_device,
            &vertices,
            copy_queue,
            copy_command_pool,
        )?;
        let( index_buffer, index_buffer_memory )= Self::create_index_buffer(
            instance,
            &device,
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
            min_pos,
            max_pos,
            device,
        })
    }

    pub fn new_from_raw_data(
        instance: &Instance,
        device: Arc<Device>,
        physical_device: vk::PhysicalDevice,
        copy_queue: vk::Queue,
        copy_command_pool: vk::CommandPool,
        vertices : Vec<Vertex>,
        indices: Vec<u32>
    ) -> Result<Self, Box<dyn Error>>  {
        let (vertex_buffer, vertex_buffer_memory)= Self::create_vertex_buffer(
            instance,
            &device,
            physical_device,
            &vertices,
            copy_queue,
            copy_command_pool,
        )?;
        let( index_buffer, index_buffer_memory )= Self::create_index_buffer(
            instance,
            &device,
            physical_device,
            &indices,
            copy_queue,
            copy_command_pool,
        )?;
        let index_count = indices.len() as u32;
        
        let mut min_pos = vertices[0].pos;
        let mut max_pos = vertices[0].pos;
        
        vertices.iter().for_each(|v| {
            min_pos.x =  min_pos.x.min(v.pos.x);
            min_pos.y =  min_pos.y.min(v.pos.y);
            min_pos.z =  min_pos.z.min(v.pos.z);

            max_pos.x =  max_pos.x.max(v.pos.x);
            max_pos.y =  max_pos.y.max(v.pos.y);
            max_pos.z =  max_pos.z.max(v.pos.z);
        });

        Ok(Self{
            index_count,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            min_pos,
            max_pos,
            device,
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

impl Drop for Model {
    fn drop(&mut self) {
        unsafe { self.device.destroy_buffer(self.index_buffer, None) }
        unsafe { self.device.free_memory(self.index_buffer_memory, None) }
        unsafe { self.device.destroy_buffer(self.vertex_buffer, None) }
        unsafe { self.device.free_memory(self.vertex_buffer_memory, None) }
    }
}

fn load_model(
    path: &str, 
    correction: nalgebra::Matrix4<f32>,
) -> Result<(Vec<Vertex>, Vec<u32>, Vector3<f32>, Vector3<f32>), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(path)?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { triangulate: true, ..Default::default() },
        |_| Ok(Default::default()),
    )?;

    let mut vertices = vec![];
    let mut indices = vec![];
    let first_index = models[0].mesh.indices[0] as usize;
    let first_pos = correction.transform_vector(&Vector3::new (
        models[0].mesh.positions[3 * first_index],
        models[0].mesh.positions[3 * first_index + 1],
        models[0].mesh.positions[3 * first_index + 2],
    ));
    let mut min_pos : Vector3<f32> = first_pos;
    let mut max_pos : Vector3<f32> = first_pos;
    
    for model in &models {

        for i in 0..model.mesh.indices.len() {
            let pi = model.mesh.indices[i] as usize;
            let pos = correction.transform_vector(&Vector3::new (
                model.mesh.positions[3 * pi],
                model.mesh.positions[3 * pi + 1],
                model.mesh.positions[3 * pi + 2],
            ));
            min_pos.x =  min_pos.x.min(pos.x);
            min_pos.y =  min_pos.y.min(pos.y);
            min_pos.z =  min_pos.z.min(pos.z);

            max_pos.x =  max_pos.x.max(pos.x);
            max_pos.y =  max_pos.y.max(pos.y);
            max_pos.z =  max_pos.z.max(pos.z);
            
            let ti = model.mesh.texcoord_indices[i] as usize;
            let ni = model.mesh.normal_indices[i] as usize;
            let vertex = Vertex {

                pos,
                
                normal: correction.transform_vector(&Vector3::new (
                    model.mesh.positions[3 * ni],
                    model.mesh.positions[3 * ni + 1],
                    model.mesh.positions[3 * ni + 2],
                )),
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



    Ok((vertices, indices, min_pos, max_pos))
}