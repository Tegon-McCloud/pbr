
pub use gltf::Gltf;
pub use gltf::Glb;

use itertools::izip;
use nalgebra::Point2;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;

use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::io::{Result, Error, ErrorKind};

use super::Loader;
use crate::material::LambertianMaterial;
use crate::material::Material;
use crate::scene::{SceneBuilder, Node, Mesh, Vertex};
use crate::spectrum::Spectrum;
use crate::texture::FactoredTexture;
use crate::texture::Texture;

use nalgebra::{Point3, Matrix4, Quaternion, convert, try_convert, Translation3, UnitQuaternion, Scale3, Affine3};
struct GltfData(Vec<gltf::buffer::Data>, Vec<gltf::image::Data>);

impl Loader for Gltf {
    fn load_from_file(path: &Path) -> Result<SceneBuilder> {

        let (document, buffers, images) = gltf::import(path).map_err(|_| Error::from(ErrorKind::InvalidData))?;
        let data = GltfData(buffers, images);

        let gltf_scene = document.default_scene().unwrap();

        let children = gltf_scene.nodes()
            .map(|gltf_node| make_node(gltf_node, &data))
            .collect();

        let root = Node { children, ..Default::default() };

        Ok(SceneBuilder { root, ..Default::default() })
    }

    fn load_from_reader<R: Read + Seek>(_rdr: &mut R) -> Result<SceneBuilder> {
        unimplemented!()
    }
}

fn make_node(gltf_node: gltf::Node, data: &GltfData) -> Node {

    let transform = make_affine(&gltf_node.transform());

    let meshes = gltf_node.mesh().iter()
        .flat_map(|gltf_mesh| gltf_mesh.primitives())
        .map(|gltf_prim| make_mesh(gltf_prim, &data))
        .collect();

    let children = gltf_node.children()
        .map(|gltf_child| make_node(gltf_child, data))
        .collect();

    Node { transform, meshes, children, }
}

fn make_affine(gltf_transform: &gltf::scene::Transform) -> Affine3<f32> {
    match gltf_transform {
        gltf::scene::Transform::Matrix{matrix} => {
            try_convert(Matrix4::from_fn(|i, j| matrix[j][i])).unwrap()
        },
        gltf::scene::Transform::Decomposed{translation: t, rotation: r, scale: s} => {
            let translation = Translation3::new(t[0], t[1], t[2]);
            let rotation = UnitQuaternion::new_unchecked(Quaternion::new(r[3],r[0], r[1], r[2]));
            let scale: Affine3<f32> = convert(Scale3::new(s[0], s[1], s[2]));
            translation * rotation * scale
        }
    }
}

fn make_mesh(gltf_prim: gltf::Primitive, data: &GltfData) -> Mesh {
    let reader = gltf_prim.reader(|buffer| Some(&data.0[buffer.index()]));

    let indices = reader.read_indices()
        .unwrap()
        .into_u32()
        .collect();

    let vertices = izip!(
        reader.read_positions().unwrap(),
        reader.read_normals().unwrap(),
        reader.read_tangents().unwrap(),
        reader.read_tex_coords(0).unwrap().into_f32(),
    )
        .map(|(p, n, t, uv)| Vertex {
            position: Point3::from(p),
            normal: Vector3::from(n),
            tangent: Vector4::from(t).xyz() * t[3],
            tex_coords: Point2::from(uv),
        })
        .collect();

    let material = make_material(gltf_prim.material(), data);
    //let material = Box::new(LambertianMaterial::flat(&Vector3::from_element(1.0)));
    Mesh { indices, vertices, material }
}

fn make_material(gltf_material: gltf::Material, data: &GltfData) -> Box<dyn Material> {

    let pmr = gltf_material.pbr_metallic_roughness();

    let base_color_factor = Spectrum::from(&pmr.base_color_factor()[0..3]);
    let metal_rough_factor = Vector2::new(pmr.metallic_factor(), pmr.roughness_factor());
    
    let base_color = FactoredTexture::new(
        base_color_factor, 
        pmr.base_color_texture().map(|info| make_texture(info.texture(), data)),
    );

    
    let material: Box<dyn Material> = Box::new(LambertianMaterial::new(base_color));

    // if metal_rough_factor.x == 0.0 {
    //     material = Box::new(LambertianMaterial::new(&base_color_factor, base_color_texture));
    // }  else {
    //     material = Box::new(MetalMaterial::new(&base_color_factor));
    // }

    // let material = Box::new(
    //     GltfMaterial::new(
    //         base_color_factor,
    //         metal_rough_factor,
    //         None,
    //         None
    //     )
    // );

    // if let Some(base_color_texture) = pmr.base_color_texture() {
    //     let base_color_texture = make_texture(base_color_texture.texture(), data);

    //     material = Box::new(GltfMaterial::textured_with_factor(&base_color_factor, base_color_texture));
    // } else {
    //     material = Box::new(LambertianMaterial::flat(&base_color_factor));
    // }

    material
}

fn make_texture(gtlf_texture: gltf::Texture, data: &GltfData) -> Texture<Spectrum<f32>> {
    let img_data = &data.1[gtlf_texture.source().index()];

    let pixels = &img_data.pixels;
    let width = img_data.width;
    let height = img_data.height;

    match img_data.format {
        gltf::image::Format::R8G8B8 => Texture::<Spectrum<f32>>::from_raw_data::<u8, 3>(width, height, pixels).unwrap(),
        gltf::image::Format::R8G8B8A8 => Texture::<Spectrum<f32>>::from_raw_data::<u8, 4>(width, height, pixels).unwrap(),
        _ => todo!(),
    }
}
