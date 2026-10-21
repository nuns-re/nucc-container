use nucc_chunk::{
    NuccChunkAmbient, NuccChunkLayerSet, NuccChunkLightDirc, NuccChunkLightPoint,
    NuccChunkLightSet, NuccChunkModelHit, NuccChunkModelPrimitiveBatch, NuccChunkModelVertex,
    NuccChunkMorphModel, NuccChunkMorphPrimitive, NuccChunkPostFilter, NuccChunkPrimitiveVertex,
    NuccChunkType, NuccChunkTypeExt,
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
fn primitive_vertex_round_trip() {
    let mut data = Vec::new();
    for value in [1u32, 0, 8, 2] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    data.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);

    let (_, boxed) =
        NuccChunkType::read_data(data.clone(), "nuccChunkPrimitiveVertex", 0x79).unwrap();
    let vertex = boxed.downcast::<NuccChunkPrimitiveVertex>().unwrap();
    assert_eq!(vertex.primitive_type, 1);
    assert_eq!(vertex.vertex_format, 0);
    assert_eq!(vertex.vertex_stride, 8);
    assert_eq!(vertex.vertex_count, 2);
    assert_eq!(vertex.data.len(), 16);
    assert_eq!(NuccChunkType::write_data(vertex, 0x79).unwrap(), data);
}

#[test]
fn model_primitive_batch_round_trip() {
    let mut data = Vec::new();
    for value in [2u32, 5, 17, 0] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    for value in [2u16, 1, 0, 0, 0, 0x0b00] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    data.extend_from_slice(&0u32.to_be_bytes());
    for value in [32u32, 0x1f003, 0] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    for value in [12u32, 2424] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    for (coord_id, vertex_num, index) in [(0u16, 14u16, 2424u32), (0, 16, 2424)] {
        data.extend_from_slice(&coord_id.to_be_bytes());
        data.extend_from_slice(&vertex_num.to_be_bytes());
        data.extend_from_slice(&index.to_be_bytes());
    }

    let (_, boxed) =
        NuccChunkType::read_data(data.clone(), "nuccChunkModelPrimitiveBatch", 0x79).unwrap();
    let batch = boxed.downcast::<NuccChunkModelPrimitiveBatch>().unwrap();
    assert_eq!(batch.coord_index, 2);
    assert_eq!(batch.primitive_vertex_index, 17);
    assert_eq!(batch.batch_count, 2);
    assert_eq!(batch.shared, [32, 0x1f003, 0]);
    assert_eq!(batch.optional, Some([12, 2424]));
    assert_eq!(batch.batches[1].vertex_num, 16);
    assert_eq!(NuccChunkType::write_data(batch, 0x79).unwrap(), data);
}

#[test]
fn morph_primitive_round_trip() {
    let mut data = Vec::new();
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&7u32.to_be_bytes());
    data.extend_from_slice(&9u32.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes());
    for (target, weight, reserved) in [(10u32, 0.25f32, 0u32), (11, 0.75, 0)] {
        data.extend_from_slice(&target.to_be_bytes());
        data.extend_from_slice(&f32be(weight));
        data.extend_from_slice(&reserved.to_be_bytes());
    }

    let (_, boxed) =
        NuccChunkType::read_data(data.clone(), "nuccChunkMorphPrimitive", 0x79).unwrap();
    let morph = boxed.downcast::<NuccChunkMorphPrimitive>().unwrap();
    assert_eq!(morph.target_num, 2);
    assert_eq!(morph.attribute, 7);
    assert_eq!(morph.model_primitive_base_index, 9);
    assert_eq!(morph.targets[1].weight, 0.75);
    assert_eq!(NuccChunkType::write_data(morph, 0x79).unwrap(), data);
}

#[test]
fn post_filter_round_trip() {
    let mut data = 3u16.to_be_bytes().to_vec();
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(&f32be(0.5));
    data.extend_from_slice(&f32be(2.0));

    let (_, boxed) = NuccChunkType::read_data(data.clone(), "nuccChunkPostFilter", 0x79).unwrap();
    let filter = boxed.downcast::<NuccChunkPostFilter>().unwrap();
    assert_eq!(filter.filter_type, 3);
    assert_eq!(filter.params, [0.5, 2.0]);
    assert_eq!(NuccChunkType::write_data(filter, 0x79).unwrap(), data);
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
