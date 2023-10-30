use crate::edgeplan::*;
use crate::pixel::*;
use crate::pixel_programs::*;
use crate::scanplan::PixelScanPlanner;

type SimpleSpriteProgram<TPixel> = BasicSpriteProgram<TPixel, Box<dyn EdgeDescriptor>, PixelScanPlanner<Box<dyn EdgeDescriptor>>>;

///
/// The standard set of pixel programs for a canvas drawing
///
pub struct CanvasPixelPrograms<TPixel, const N: usize>
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    /// The main program cache
    pub (super) program_cache: PixelProgramCache<TPixel>,

    /// The basic solid colour pixel program
    pub (super) solid_color: StoredPixelProgramFromProgram<SolidColorProgram<TPixel>>,

    /// The 'source over' alpha blending pixel program
    pub (super) source_over_color: StoredPixelProgramFromProgram<SourceOverColorProgram<TPixel>>,

    /// The general solid colour blending pixel program
    pub (super) blend_color: StoredPixelProgramFromProgram<BlendColorProgram<TPixel>>,

    /// The basic texture rendering program
    pub (super) basic_texture: StoredPixelProgramFromProgram<BasicTextureProgram<TPixel, RgbaTexture>>,

    /// The basic sprite rendering program (can scale or transform the sprite, and will render it as source over with 100% transparency)
    pub (super) basic_sprite: StoredPixelProgramFromProgram<SimpleSpriteProgram<TPixel>>,
}

impl<TPixel, const N: usize> Default for CanvasPixelPrograms<TPixel, N> 
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    fn default() -> Self {
        let mut cache       = PixelProgramCache::empty();
        let solid_color     = cache.add_pixel_program(SolidColorProgram::default());
        let source_over     = cache.add_pixel_program(SourceOverColorProgram::default());
        let blend_color     = cache.add_pixel_program(BlendColorProgram::default());
        let basic_texture   = cache.add_pixel_program(BasicTextureProgram::default());
        let basic_sprite    = cache.add_pixel_program::<SimpleSpriteProgram<TPixel>>(BasicSpriteProgram::default());

        CanvasPixelPrograms { 
            program_cache:      cache, 
            solid_color:        solid_color,
            source_over_color:  source_over,
            blend_color:        blend_color,
            basic_texture:      basic_texture,
            basic_sprite:       basic_sprite,
        }
    }
}

impl<TPixel, const N: usize> CanvasPixelPrograms<TPixel, N> 
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    ///
    /// Creates the pixel program data cache to use with the pixel programs
    ///
    #[inline]
    pub fn create_data_cache(&mut self) -> PixelProgramDataCache<TPixel> {
        self.program_cache.create_data_cache()
    }
}
