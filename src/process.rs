use image::GenericImageView;
use oxipng::optimize_from_memory;

pub fn process(img_bytes: Vec<u8>, tile: bool, relayer: bool) -> Vec<u8> {
	let img = image::load_from_memory(&img_bytes).expect("failed to parse image data");

	let cull = !img.pixels().any(|pixel| (1..255).contains(&pixel.2.0[3]));

	let mut scaled = if tile {
		scale_tiled(&img)
	} else {
		scale_notile(&img)
	};

	if cull {
		for pixel in scaled.pixels_mut() {
			pixel.0[3] = if pixel.0[3] < 191 { 0 } else { 255 };
		}
	}

	if relayer {
		for pixel in scaled.enumerate_pixels_mut() {
			// if pixel opacity is 0, overwrite with data from original
			if pixel.2.0[3] != 0 {
				continue;
			}
			pixel.2.0 = img.get_pixel(pixel.0 / 4, pixel.1 / 4).0;
		}
	}

	let mut scaled_png: Vec<u8> = Vec::new();
	scaled
		.write_to(
			&mut std::io::Cursor::new(&mut scaled_png),
			image::ImageFormat::Png,
		)
		.expect("failed to convert scaled image to png");

	optimize_from_memory(
		&scaled_png,
		&oxipng::Options {
			optimize_alpha: true,
			strip: oxipng::StripChunks::All,
			..Default::default()
		},
	)
	.expect("failed to optimize")
}

fn scale_notile(img: &image::DynamicImage) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
	let width = img.width();
	let height = img.height();
	let out_rgba = xbrz::scale_rgba(&img.to_rgba8(), width as usize, height as usize, 4);
	image::RgbaImage::from_vec(width * 4, height * 4, out_rgba).unwrap()
}

fn scale_tiled(img: &image::DynamicImage) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
	let width = img.width();
	let height = img.height();

	let mut tiled = image::RgbaImage::new(width * 3, height * 3);
	image::imageops::tile(&mut tiled, img);
	tiled =
		image::imageops::crop(&mut tiled, width - 4, height - 4, width + 8, height + 8).to_image();
	let tiled_width = tiled.width();
	let tiled_height = tiled.height();

	let out_rgba = xbrz::scale_rgba(
		tiled.as_raw(),
		tiled_width as usize,
		tiled_height as usize,
		4,
	);

	image::imageops::crop(
		&mut image::RgbaImage::from_vec(tiled_width * 4, tiled_height * 4, out_rgba).unwrap(),
		16,
		16,
		width * 4,
		height * 4,
	)
	.to_image()
}
