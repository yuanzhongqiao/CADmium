use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use truck_meshalgo::prelude::OptimizingFilter;
use truck_meshalgo::tessellation::MeshableShape;
use truck_meshalgo::tessellation::MeshedShape;
// use truck_polymesh::cgmath::Point3 as TruckPoint3;

use crate::project::Point3;
use crate::project::Project;
use crate::project::RealPlane;
use crate::project::RealSketch;
use crate::project::Vector3;
use crate::sketch::arc_to_points;
use crate::sketch::Ring;
use crate::sketch::Segment;
use crate::sketch::Sketch;
use crate::sketch::Vector2;

// use truck_meshalgo::prelude::*;
use truck_modeling::{
    builder, Edge, Face as TruckFace, Point3 as TruckPoint3, Vector3 as TruckVector3, Vertex, Wire,
};

const MESH_TOLERANCE: f64 = 0.001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrusion {
    pub sketch_name: String,
    pub face_ids: Vec<u64>,
    pub length: f64,
    pub offset: f64,
    pub direction: Vector3,
    // TODO: add a "mode" field for "new" vs "add" vs "remove"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid {
    pub name: String,
    pub vertices: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub uvs: Vec<Vector2>,
    pub indices: Vec<usize>,
    pub triangles: Vec<Vec<u64>>,
}

impl Solid {
    pub fn from_extrusion(
        name: String,
        plane: &RealPlane,
        sketch: &RealSketch,
        extrusion: &Extrusion,
    ) -> HashMap<String, Self> {
        let mut retval = HashMap::new();

        let extrusion_vector = extrusion.direction.times(extrusion.length);
        let vector = TruckVector3::new(extrusion_vector.x, extrusion_vector.y, extrusion_vector.z);

        for face_id in &extrusion.face_ids {
            let face = sketch.faces.get(*face_id as usize).unwrap();

            // println!("face: {:?}", face);
            let exterior = &face.exterior;
            let mut wires: Vec<Wire> = Vec::new();

            // the exterior wire comes first
            wires.push(Self::to_wire(plane, sketch, extrusion, exterior));

            // then the interior wires
            for interior in &face.holes {
                wires.push(Self::to_wire(plane, sketch, extrusion, interior));
            }

            let face = builder::try_attach_plane(&wires).unwrap();
            // println!("A Face: {:?}", face);

            let truck_solid = builder::tsweep(&face, vector);
            // println!("A Solid: {:?}", truck_solid);

            let mut solid = Solid {
                name: format!("{}:{}", name, face_id),
                vertices: vec![],
                normals: vec![],
                triangles: vec![],
                uvs: vec![],
                indices: vec![],
            };

            let mut mesh = truck_solid.triangulation(MESH_TOLERANCE).to_polygon();
            mesh.put_together_same_attrs();

            // the mesh is prepared for obj export, but we need to convert it
            // to a format compatible for rendering
            // We have to brute force this. Go through every single triangle
            // and emit three positions, three normals, and three uvs.
            let mut index = 0 as usize;
            for face in mesh.tri_faces() {
                for v in face.iter() {
                    let vertex_index = v.pos;
                    let normal_index = v.nor.unwrap();
                    let uv_index = v.uv.unwrap();
                    let vertex = mesh.positions()[vertex_index];
                    let normal = mesh.normals()[normal_index];
                    let uv = mesh.uv_coords()[uv_index];

                    let pt = Vector3::new(vertex.x, vertex.y, vertex.z);
                    solid.vertices.push(pt);
                    solid
                        .normals
                        .push(Vector3::new(normal.x, normal.y, normal.z));
                    solid.uvs.push(Vector2::new(uv.x, uv.y));
                    solid.indices.push(index);

                    index += 1;
                }
            }

            retval.insert(format!("{}:{}", name, face_id), solid);
        }

        retval
    }

    pub fn to_wire(
        plane: &RealPlane,
        sketch: &RealSketch,
        extrusion: &Extrusion,
        exterior: &Ring,
    ) -> Wire {
        match exterior {
            Ring::Circle(circle) => {
                println!("circle: {:?}", circle);

                let center = sketch.points.get(&circle.center).unwrap();
                let center_vertex = builder::vertex(TruckPoint3::new(center.x, center.y, center.z));

                let wire = Wire::new();
                wire
            }
            Ring::Segments(segments) => {
                println!("segments: {:?}", segments);
                // let mut builder = builder::FaceBuilder::new();
                let mut vertices: HashMap<u64, Vertex> = HashMap::new();

                // First just collect all the points as Truck Vertices
                // This is important because for shapes to be closed,
                // Truck requires that the start point IS the end point
                for segment in segments.iter() {
                    match segment {
                        Segment::Line(line) => {
                            let start = sketch.points.get(&line.start).unwrap();
                            let start_vertex =
                                builder::vertex(TruckPoint3::new(start.x, start.y, start.z));
                            let end = sketch.points.get(&line.end).unwrap();
                            let end_vertex = builder::vertex(TruckPoint3::new(end.x, end.y, end.z));
                            vertices.insert(line.start, start_vertex);
                            vertices.insert(line.end, end_vertex);
                        }
                        Segment::Arc(arc) => {
                            let start = sketch.points.get(&arc.start).unwrap();
                            let start_vertex =
                                builder::vertex(TruckPoint3::new(start.x, start.y, start.z));
                            let end = sketch.points.get(&arc.end).unwrap();
                            let end_vertex = builder::vertex(TruckPoint3::new(end.x, end.y, end.z));
                            let center = sketch.points.get(&arc.center).unwrap();
                            let center_vertex =
                                builder::vertex(TruckPoint3::new(center.x, center.y, center.z));
                            vertices.insert(arc.start, start_vertex);
                            vertices.insert(arc.end, end_vertex);
                            vertices.insert(arc.center, center_vertex);
                        }
                    }
                }

                let mut edges: Vec<Edge> = Vec::new();
                // Now add the segments to the wire
                for segment in segments.iter() {
                    match segment {
                        Segment::Line(line) => {
                            let start_vertex = vertices.get(&line.start).unwrap();
                            let end_vertex = vertices.get(&line.end).unwrap();
                            let edge = builder::line(start_vertex, end_vertex);
                            edges.push(edge);
                        }
                        Segment::Arc(arc) => {
                            let start_point = sketch.points.get(&arc.start).unwrap();
                            let end_point = sketch.points.get(&arc.end).unwrap();
                            let center_point = sketch.points.get(&arc.center).unwrap();
                            let transit = find_transit(
                                plane,
                                start_point,
                                end_point,
                                center_point,
                                arc.clockwise,
                            );

                            let start_vertex = vertices.get(&arc.start).unwrap();
                            let end_vertex = vertices.get(&arc.end).unwrap();
                            let transit_point = TruckPoint3::new(transit.x, transit.y, transit.z);

                            // center point is not a vertex, but a point
                            let center_point = sketch.points.get(&arc.center).unwrap();
                            let edge = builder::circle_arc(start_vertex, end_vertex, transit_point);
                            edges.push(edge);
                        }
                    }
                }

                let wire = Wire::from_iter(edges.into_iter());
                wire
            }
        }
    }
}

pub fn find_transit(
    real_plane: &RealPlane,
    start: &Point3,
    end: &Point3,
    center: &Point3,
    clockwise: bool,
) -> Point3 {
    let radius = start.distance_to(center);

    let start = real_plane.plane.project(start);
    let end = real_plane.plane.project(end);
    let center = real_plane.plane.project(center);

    let pts = arc_to_points(&start, &end, &center, clockwise);

    let transit = &pts[pts.len() / 2];

    let transit_3d = real_plane.plane.unproject(&transit);
    transit_3d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project_solid() {
        let mut p = Project::new("Test Extrusion");
        p.add_defaults();

        // now get solids? save as obj or stl or step?
        let mut workbench = p.workbenches.get_mut(0).unwrap();
        let realization = workbench.realize(100);
        let solids = realization.solids;
        // println!("solids: {:?}", solids);
    }
}