mod process;
use process::process;

fn main() {
    let in_image = image::open("crafting_table_front.png").expect("you should exist bud");
    let out_image = process(in_image, true, false);

    out_image.save("scaled.png").unwrap();
}
