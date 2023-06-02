#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

pub mod context;
pub mod gui;

use image::{ImageBuffer, Rgba};
use log::*;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::CopyImageToBufferInfo;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::{
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage,
    },
    image::StorageImage,
    pipeline::ComputePipeline,
    sync::{self, GpuFuture},
};

use std::sync::Arc;

use crate::shaders;

pub struct Renderer {
    pub context: context::Context,
    pub gui: gui::GUI,
    pub pipelines: Vec<Arc<ComputePipeline>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        let context = context::Context::new();
        let gui = gui::GUI::new();
        let mut pipelines = Vec::new();
        pipelines.push(
            ComputePipeline::new(
                context.device.clone(),
                shaders::test::load(context.device.clone())
                    .expect("RENDERER | failed to load test shader")
                    .entry_point("main")
                    .unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
            .clone(),
        );
        Renderer {
            context,
            gui,
            pipelines,
        }
    }
    pub fn test(&self) {
        let image = StorageImage::new(
            &self.context.allocator.clone(),
            vulkano::image::ImageDimensions::Dim2d {
                width: 640,
                height: 640,
                array_layers: 1,
            },
            Format::R8G8B8A8_UNORM,
            Some(self.context.queue.clone().queue_family_index()),
        )
        .unwrap();
        let view = ImageView::new_default(image.clone()).unwrap();
        let layout = self.pipelines[0].layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.context.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::image_view(0, view.clone())],
        )
        .unwrap();
        let buf = Buffer::from_iter(
            &self.context.allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Download,
                ..Default::default()
            },
            (0..640 * 640 * 4).map(|_| 0u8),
        )
        .expect("TEST | failed to create image buffer");
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            self.context.device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            self.context.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        let work_group_counts = [640, 640, 1];

        command_buffer_builder
            .bind_pipeline_compute(self.pipelines[0].clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipelines[0].layout().clone(),
                0,
                set,
            )
            .dispatch(work_group_counts)
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                image.clone(),
                buf.clone(),
            ))
            .unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = sync::now(self.context.device.clone())
            .then_execute(self.context.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        let buffer_content = buf.read().unwrap();
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(640, 640, &buffer_content[..]).unwrap();
        image.save("test.png").unwrap();
    }
}
