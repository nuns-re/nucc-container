use nucc_chunk::{
    NuccChunkAmbient, NuccChunkLayerSet, NuccChunkLightDirc, NuccChunkLightPoint,
    NuccChunkLightSet, NuccChunkModelHit, NuccChunkModelVertex, NuccChunkMorphModel, NuccChunkType,
    NuccChunkTypeExt,
};

fn f32be(value: f32) -> [u8; 4] {
    value.to_be_bytes()
}

#[test]
fn ambient_color_round_trip() {
    let mut data = Vec::new();
    for value in [0.25, 0.5, 0.75, 1.0] {
        data.extend_from_slice(&f32be(value));
    }
    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkAmbient", 0x7a).unwrap();
    let ambient = boxed.downcast::<NuccChunkAmbient>().unwrap();
    assert_eq!(ambient.color, [0.25, 0.5, 0.75, 1.0]);
    assert_eq!(NuccChunkType::write_data(ambient, 0x7a).unwrap(), data);
}

#[test]
fn light_dirc_round_trip() {
    let mut data = vec![0u8; 0x40];
    data[0..4].copy_from_slice(&12u32.to_be_bytes());
    for (index, value) in [0.1f32, 0.2, 0.3].into_iter().enumerate() {
        let offset = 0x10 + index * 4;
        data[offset..offset + 4].copy_from_slice(&f32be(value));
    }
    data[0x1c..0x20].copy_from_slice(&f32be(2.0));
    for (index, value) in [0.0f32, 0.0, 1.0, 0.0].into_iter().enumerate() {
        let offset = 0x30 + index * 4;
        data[offset..offset + 4].copy_from_slice(&f32be(value));
    }

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkLightDirc", 0x7a).unwrap();
    let light = boxed.downcast::<NuccChunkLightDirc>().unwrap();
    assert_eq!(light.index, 12);
    assert_eq!(light.color, [0.1, 0.2, 0.3]);
    assert_eq!(light.intensity, 2.0);
    assert_eq!(light.direction, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(NuccChunkType::write_data(light, 0x7a).unwrap(), data);
}

#[test]
fn light_point_round_trip() {
    let mut data = vec![0u8; 0x40];
    data[0..4].copy_from_slice(&3u32.to_be_bytes());
    data[0x10..0x14].copy_from_slice(&f32be(1.0));
    data[0x14..0x18].copy_from_slice(&f32be(0.5));
    data[0x18..0x1c].copy_from_slice(&f32be(0.25));
    data[0x1c..0x20].copy_from_slice(&f32be(0.0));
    data[0x20..0x24].copy_from_slice(&f32be(4.0));
    data[0x24..0x28].copy_from_slice(&f32be(10.0));
    data[0x28..0x2c].copy_from_slice(&f32be(20.0));
    data[0x2c..0x30].copy_from_slice(&f32be(30.0));
    data[0x30..0x34].copy_from_slice(&f32be(40.0));
    data[0x34..0x38].copy_from_slice(&f32be(50.0));

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkLightPoint", 0x7a).unwrap();
    let light = boxed.downcast::<NuccChunkLightPoint>().unwrap();
    assert_eq!(light.index, 3);
    assert_eq!(light.color, [1.0, 0.5, 0.25]);
    assert_eq!(light.intensity, 4.0);
    assert_eq!(light.position, [10.0, 20.0, 30.0]);
    assert_eq!(light.radius, 40.0);
    assert_eq!(light.falloff, 50.0);
    assert_eq!(NuccChunkType::write_data(light, 0x7a).unwrap(), data);
}

#[test]
fn layer_set_round_trip() {
    let mut data = vec![1, 1, 0, 8, 0, 3];
    data.extend_from_slice(&[0, 0, 0, 0, 0, 2, 0, 1]);
    data.extend_from_slice(&[0, 0, 0, 4, 0, 2, 0, 6]);
    data.extend_from_slice(b"a\0bb\0ccc");

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkLayerSet", 0x7a).unwrap();
    let layer_set = boxed.downcast::<NuccChunkLayerSet>().unwrap();
    assert_eq!(layer_set.layer_count, 1);
    assert_eq!(layer_set.selector_count, 1);
    assert_eq!(layer_set.layers[0].priority, 1);
    assert_eq!(
        layer_set.string_at(layer_set.layers[0].name_offset),
        Some("a")
    );
    assert_eq!(layer_set.string_at(2), Some("bb"));
    assert_eq!(NuccChunkType::write_data(layer_set, 0x7a).unwrap(), data);
}

#[test]
fn morph_model_round_trip() {
    let mut data = vec![0, 2, 0, 0];
    data.extend_from_slice(&10u32.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&11u32.to_be_bytes());
    data.extend_from_slice(&f32be(0.5));
    data.extend_from_slice(&12u32.to_be_bytes());
    data.extend_from_slice(&f32be(1.0));

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkMorphModel", 0x7a).unwrap();
    let morph = boxed.downcast::<NuccChunkMorphModel>().unwrap();
    assert_eq!(morph.target_num, 2);
    assert_eq!(morph.model_base_index, 10);
    assert_eq!(morph.targets.len(), 2);
    assert_eq!(morph.targets[0].vertex_target_index, 11);
    assert_eq!(morph.targets[0].weight, 0.5);
    assert_eq!(NuccChunkType::write_data(morph, 0x7a).unwrap(), data);
}

#[test]
fn light_set_round_trip() {
    let mut data = vec![1, 1, 1, 0, 0, 6, 0, 0];
    data.extend_from_slice(&[0, 0, 0, 0]);
    data.extend_from_slice(&[0, 0, 0, 2, 0, 0, 0, 5]);
    data.extend_from_slice(&[0, 0, 0, 4]);
    data.extend_from_slice(b"a\0b\0c\0");

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkLightSet", 0x7a).unwrap();
    let light_set = boxed.downcast::<NuccChunkLightSet>().unwrap();
    assert_eq!(light_set.categories.len(), 1);
    assert_eq!(light_set.category_groups.len(), 1);
    assert_eq!(light_set.category_groups[0].category, 5);
    assert_eq!(light_set.modes.len(), 1);
    assert_eq!(light_set.string_at(2), Some("b"));
    assert_eq!(NuccChunkType::write_data(light_set, 0x7a).unwrap(), data);
}

#[test]
fn model_vertex_round_trip() {
    let mut data = 8u32.to_be_bytes().to_vec();
    data.extend_from_slice(b"data5678");

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkModelVertex", 0x7a).unwrap();
    let model = boxed.downcast::<NuccChunkModelVertex>().unwrap();
    assert_eq!(model.data_size, 8);
    assert_eq!(model.data, b"data5678");
    assert_eq!(NuccChunkType::write_data(model, 0x7a).unwrap(), data);
}

fn push_model_hit_triangle(data: &mut Vec<u8>, v: [[f32; 3]; 3]) {
    for point in v {
        for coordinate in point {
            data.extend_from_slice(&f32be(coordinate));
        }
    }
}

#[test]
fn model_hit_v121_round_trip() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&0x3067u32.to_be_bytes());
    push_model_hit_triangle(
        &mut data,
        [[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [0.0, 100.0, 0.0]],
    );

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkModelHit", 0x79).unwrap();
    let hit = boxed.downcast::<NuccChunkModelHit>().unwrap();
    assert_eq!(hit.objects.len(), 1);
    assert_eq!(hit.objects[0].poly_count, 1);
    assert_eq!(hit.objects[0].attr_a, 0x3067);
    assert_eq!(hit.triangles.len(), 1);
    assert_eq!(hit.triangles[0].v[1], [100.0, 0.0, 0.0]);
    assert_eq!(NuccChunkType::write_data(hit, 0x79).unwrap(), data);
}

#[test]
fn model_hit_v122_round_trip() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&0x606064u32.to_be_bytes());
    data.extend_from_slice(&0x12345678u32.to_be_bytes());
    data.extend_from_slice(&0x9abcdef0u32.to_be_bytes());
    push_model_hit_triangle(
        &mut data,
        [[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [0.0, 100.0, 0.0]],
    );

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkModelHit", 0x7a).unwrap();
    let hit = boxed.downcast::<NuccChunkModelHit>().unwrap();
    assert_eq!(hit.objects[0].attr_b, 0x12345678);
    assert_eq!(hit.objects[0].unused_pad, 0x9abcdef0);
    assert_eq!(NuccChunkType::write_data(hit, 0x7a).unwrap(), data);
}

#[test]
fn new_chunk_reference_stubs_are_typed() {
    let data = vec![0, 0, 0, 4, 0, 0, 0, 0];
    for (name, chunk) in [
        (
            "nuccChunkLightSet",
            NuccChunkType::read_data(data.clone(), "nuccChunkLightSet", 0x7a)
                .unwrap()
                .1,
        ),
        (
            "nuccChunkModelHit",
            NuccChunkType::read_data(data.clone(), "nuccChunkModelHit", 0x7a)
                .unwrap()
                .1,
        ),
        (
            "nuccChunkModelVertex",
            NuccChunkType::read_data(data.clone(), "nuccChunkModelVertex", 0x7a)
                .unwrap()
                .1,
        ),
    ] {
        assert!(!chunk.is_opaque(), "{name}");
        assert_eq!(chunk.chunk_type().as_str(), name);
    }
}
