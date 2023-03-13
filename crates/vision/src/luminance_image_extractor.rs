use std::{num::NonZeroU32, time::Instant};

use color_eyre::Result;
use context_attribute::context;
use fast_image_resize::{DynamicImageView, FilterType, ImageView, ResizeAlg, Resizer};
use framework::{AdditionalOutput, MainOutput};
use ittapi::{Task, Domain};
use types::{grayscale_image::GrayscaleImage, nao_image::NaoImage};

use crate::CyclerInstance;

pub struct LuminanceImageExtractor {}

#[context]
pub struct CreationContext {}

#[context]
pub struct CycleContext {
    pub instance: CyclerInstance,
    pub image: Input<NaoImage, "image">,
}

#[context]
pub struct MainOutputs {
    pub luminance_image: MainOutput<GrayscaleImage>,
}

impl LuminanceImageExtractor {
    pub fn new(_context: CreationContext) -> Result<Self> {
        Ok(Self {})
    }

    pub fn cycle(&mut self, mut context: CycleContext) -> Result<MainOutputs> {
        let domain = Domain::new("Benchmark");
        {
            let _task = Task::begin(&domain, "extract");
            let mut grayscale_buffer: Vec<_> =
                Vec::with_capacity((context.image.width() * context.image.height()) as usize);
            grayscale_buffer.extend(
                context
                    .image
                    .buffer
                    .iter()
                    .flat_map(|pixel| [pixel.y1, pixel.y2]),
            );
        }
        {
            let _task = Task::begin(&domain, "view");
            let y_image = ImageView::from_buffer(
                NonZeroU32::new(context.image.width()).unwrap(),
                NonZeroU32::new(context.image.height()).unwrap(),
                &grayscale_buffer,
            )?;
        }
        {
            let _task = Task::begin(&domain, "small_image");
            let dst_width = NonZeroU32::new(80).unwrap();
            let dst_height = NonZeroU32::new(60).unwrap();

            let mut dst_image =
                fast_image_resize::Image::new(dst_width, dst_height, y_image.pixel_type());
        }
        {
            let _task = Task::begin(&domain, "resize");
            let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Hamming));
            resizer
                .resize(&DynamicImageView::U8(y_image), &mut dst_image.view_mut())
                .unwrap();
        }
        {
            let _task = Task::begin(&domain, "output");
            let luminance_image =
                GrayscaleImage::from_vec(dst_width.get(), dst_height.get(), dst_image.into_vec());
        }
        Ok(MainOutputs {
            luminance_image: luminance_image.into(),
        })
    }
}
