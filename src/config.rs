use crate::{hittable::HittableVec, Color, Point3, Vec3};
use miette::{bail, Result};
use owo_colors::OwoColorize;
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug)]
struct ConfigModel {
    textures: TextureStorage,
    materials: MaterialStorage,
    objects: Vec<ObjectModel>,
}

#[derive(Debug)]
enum TextureModel {
    SolidColor {
        color: Color,
    },
    Checkerboard {
        scale: f64,
        color1: Box<TextureModel>,
        color2: Box<TextureModel>,
    },
    Image {
        path: PathBuf,
    },
    Ref(TextureStorageId),
}

#[derive(Debug)]
enum MaterialModel {
    Lambertian(TextureStorageId),
    DiffuseLight(TextureStorageId),
    Isotropic(TextureStorageId),
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { refractive_index: f64 },
}

#[derive(Debug)]
enum ObjectModel {
    Sphere {
        center: Point3,
        radius: f64,
        material: MaterialStorageId,
    },
    Parallelogram {
        corner: Point3,
        // vectors across two edges
        vectors: [Vec3; 2],
        material: MaterialStorageId,
    },
    Triangle {
        points: [Point3; 3],
        material: MaterialStorageId,
    },
    Disc {
        center: Point3,
        // radial vectors
        vectors: [Vec3; 2],
        material: MaterialStorageId,
    },
}

#[derive(Debug)]
struct TextureStorage {
    textures: HashMap<String, TextureModel>,
    anonymous: Vec<TextureModel>,
}

#[derive(Debug)]
enum TextureStorageId {
    Anonymous(usize),
    Named(String),
}

type MaterialStorage = HashMap<String, MaterialModel>;

#[derive(Debug, PartialEq, Eq, Hash)]
struct MaterialStorageId(String);

impl TextureStorage {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            anonymous: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            textures: HashMap::with_capacity(cap),
            anonymous: Vec::new(),
        }
    }

    pub fn push_anon(&mut self, texture: TextureModel) -> TextureStorageId {
        self.anonymous.push(texture);
        TextureStorageId::Anonymous(self.anonymous.len())
    }

    pub fn push_named(&mut self, key: String, texture: TextureModel) -> TextureStorageId {
        self.textures.entry(key.clone()).insert_entry(texture);
        TextureStorageId::Named(key)
    }

    pub fn contains_named_key(&self, name: &str) -> bool {
        self.textures.contains_key(name)
    }
}

trait ValueExt {
    fn parse_color(&self, key: &str) -> Result<Color>;
    fn parse_floatlike(&self, key: &str) -> Result<f64>;
    fn parse_pathbuf(&self, key: &str) -> Result<PathBuf>;
    fn parse_point3(&self, key: &str) -> Result<Point3>;
    fn parse_vec3(&self, key: &str) -> Result<Vec3>;

    fn parse_array<'a, 'b>(&'a self, key: &'b str) -> Result<&'a Vec<toml::Value>>;
    fn parse_texture(&self, key: &str, storage: &TextureStorage) -> Result<TextureStorageId>;
    fn parse_material(&self, key: &str, storage: &MaterialStorage) -> Result<MaterialStorageId>;
}

impl ValueExt for toml::Value {
    fn parse_color(&self, key: &str) -> Result<Color> {
        match self {
            toml::Value::String(color_str) => {
                let sani = color_str.trim().trim_matches('#');
                let hex = u32::from_str_radix(sani, 16)
                    .map_err(|e| miette::miette!(r#"Invalid hex string "{}": {}"#, sani, e))?;
                Ok(Color::hex(hex))
            }
            toml::Value::Integer(color_int) => Ok(Color::hex((*color_int).try_into().unwrap())),
            _ => {
                bail!("{} must be a hex code or number.", key.green());
            }
        }
    }

    fn parse_floatlike(&self, key: &str) -> Result<f64> {
        match self {
            toml::Value::Float(f) => Ok(*f),
            // may be a lossy conversion
            toml::Value::Integer(i) => Ok(*i as f64),
            _ => {
                bail!("{} must be a decimal number.", key.green());
            }
        }
    }

    fn parse_pathbuf(&self, key: &str) -> Result<PathBuf> {
        match self {
            toml::Value::String(s) => Ok(PathBuf::from(s)),
            _ => {
                bail!("{} must be a valid filepath.", key.green());
            }
        }
    }

    fn parse_point3(&self, key: &str) -> Result<Point3> {
        let toml::Value::Array(arr) = self else {
            bail!(
                "{} must be a valid 3D point, represented as {}.",
                key.green(),
                "[x, y, z]".purple()
            );
        };

        if arr.len() != 3 {
            bail!(
                "{} must be a valid {} point, represented as {}.",
                key.green(),
                "3D".bold(),
                "[x, y, z]".purple()
            );
        }

        let mut res: [f64; 3] = [f64::NAN; 3];

        for i in 0..3 {
            res[i] = arr[i].parse_floatlike(&format!("{}.{}", key, i))?;
        }
        Ok(Point3::from(res))
    }

    fn parse_vec3(&self, key: &str) -> Result<Vec3> {
        let toml::Value::Array(arr) = self else {
            bail!(
                "{} must be a valid 3D vector, represented as {}.",
                key.green(),
                "[x, y, z]".purple()
            );
        };

        if arr.len() != 3 {
            bail!(
                "{} must be a valid {} vector, represented as {}.",
                key.green(),
                "3D".bold(),
                "[x, y, z]".purple()
            );
        }

        let mut res: [f64; 3] = [f64::NAN; 3];

        for i in 0..3 {
            res[i] = arr[i].parse_floatlike(&format!("{}.{}", key, i))?;
        }
        Ok(Vec3::from(Point3::from(res)))
    }

    fn parse_array<'a, 'b>(&'a self, key: &'b str) -> Result<&'a Vec<toml::Value>> {
        match self {
            toml::Value::Array(a) => Ok(a),
            _ => {
                bail!("{} must be an array.", key.green());
            }
        }
    }

    fn parse_texture(&self, key: &str, storage: &TextureStorage) -> Result<TextureStorageId> {
        match self {
            toml::Value::String(a) => {
                if !storage.contains_named_key(a) {
                    bail!(
                        help = format!("No texture with ID {} has been loaded.", a.purple()),
                        "{} does not describe a valid texture.",
                        key.green()
                    );
                }
                Ok(TextureStorageId::Named(a.to_string()))
            }
            _ => {
                bail!(
                    "{} must be a string representing a previously listed texture.",
                    key.green()
                );
            }
        }
    }

    fn parse_material(&self, key: &str, storage: &MaterialStorage) -> Result<MaterialStorageId> {
        match self {
            toml::Value::String(a) => {
                if !storage.contains_key(a) {
                    bail!(
                        help = format!("No material named {} has been loaded.", a.purple()),
                        "{} does not describe a valid texture.",
                        key.green()
                    );
                }
                Ok(MaterialStorageId(a.to_string()))
            }
            _ => {
                bail!(
                    "{} must be a string representing a previously listed texture.",
                    key.green()
                );
            }
        }
    }
}

fn require_value<'a, 'b>(
    table: &'a toml::Table,
    key: &'b str,
    parent_key: &'b str,
) -> Result<&'a toml::Value> {
    if let Some(value) = table.get(key) {
        Ok(value)
    } else {
        bail!(
            "{} must be provided.",
            format!("{}.{}", parent_key, key).green()
        );
    }
}

impl TextureModel {
    pub fn parse(
        name: &str,
        table: &toml::Table,
        texture_storage: &mut TextureStorage,
    ) -> Result<Self> {
        let Some(toml::Value::String(texture_type)) = table.get("type") else {
            bail!(
                "{} must be a string.",
                format!("config.textures.{}.type", name).green()
            );
        };

        match &texture_type.to_ascii_uppercase()[..] {
            "COLOR" | "SOLIDCOLOR" | "SOLID_COLOR" => {
                let value = require_value(table, "color", &format!("config.textures.{name}"))?;
                let color = value.parse_color(&format!("config.textures.{name}.color"))?;
                Ok(Self::SolidColor { color })
            }
            "CHECKERBOARD" | "CHECKER" => {
                let scale = require_value(table, "scale", &format!("config.textures.{name}"))?;
                let scale = scale.parse_floatlike(&format!("config.textures.{name}.scale"))?;

                /*
                # Two referenced textures
                textures = ["tex", "tex2"]
                # If one is a valid color, parse it first & convert to anonymous SolidColor texture
                textures = [0xfff, "tex2"]
                 */
                // for now, `textures` is expected to contain two color values.
                // TODO: this requirement should be relaxed.
                let textures =
                    require_value(table, "textures", &format!("config.textures.{name}"))?;
                let textures = textures.parse_array(&format!("config.textures.{name}.textures"))?;

                // TODO: relax this restriction.
                // >> blocked by the Checkerboard texture allowing more than 2 subtextures.
                if textures.len() != 2 {
                    bail!(
                        "{} must be an array of length 2.",
                        format!("config.textures.{name}.textures").green()
                    );
                }

                // construct anonymous textures
                let color =
                    textures[0].parse_color(&format!("config.textures.{name}.textures.0"))?;
                let ind1 = texture_storage.push_anon(TextureModel::SolidColor { color });
                let color =
                    textures[1].parse_color(&format!("config.textures.{name}.textures.1"))?;
                let ind2 = texture_storage.push_anon(TextureModel::SolidColor { color });

                Ok(Self::Checkerboard {
                    scale,
                    color1: Box::new(TextureModel::Ref(ind1)),
                    color2: Box::new(TextureModel::Ref(ind2)),
                })
            }
            "IMAGE" => {
                let value = require_value(table, "path", &format!("config.textures.{name}"))?;
                let path = value.parse_pathbuf(&format!("config.textures.{name}.path"))?;
                if !path.try_exists().is_ok_and(|e| e == true) {
                    bail!(miette::diagnostic!(
                        help = format!(
                            "attempted to load from {}",
                            format!("config.textures.{}.path", name).purple(),
                        ),
                        "Failed to find file {}.",
                        path.display().green(),
                    ));
                }
                Ok(Self::Image { path })
            }
            _ => {
                bail!(miette::diagnostic!(
                    help = format!(
                        "valid colors include: {}",
                        r#""color" | "checkerboard" | "image""#.purple()
                    ),
                    "{} must be a valid texture type.",
                    format!("config.textures.{}.type", name).green(),
                ));
            }
        }
    }
}

impl MaterialModel {
    pub fn parse(
        name: &str,
        table: &toml::Table,
        texture_storage: &mut TextureStorage,
    ) -> Result<Self> {
        let Some(toml::Value::String(mat_type)) = table.get("type") else {
            bail!(
                "{} must be a string.",
                format!("config.materials.{}.type", name).green()
            );
        };

        match &mat_type.to_ascii_uppercase()[..] {
            "LAMBERTIAN" => {
                let value = require_value(table, "texture", &format!("config.materials.{name}"))?;
                let texture = value
                    .parse_texture(&format!("config.materials.{name}.texture"), texture_storage)?;
                Ok(Self::Lambertian(texture))
            }
            "METAL" | "METALLIC" | "FUZZY" => {
                let value = require_value(table, "albedo", &format!("config.materials.{name}"))?;
                let albedo = value.parse_color(&format!("config.materials.{name}.albedo"))?;

                let value = require_value(table, "fuzz", &format!("config.materials.{name}"))?;
                let mut fuzz = value.parse_floatlike(&format!("config.materials.{name}.fuzz"))?;

                if fuzz > 1.0 {
                    fuzz /= 100.0;
                }
                Ok(Self::Metal { albedo, fuzz })
            }
            "LIGHT" | "LIGHTSOURCE" | "DIFFUSELIGHT" => {
                let value = require_value(table, "texture", &format!("config.materials.{name}"))?;
                let texture = value
                    .parse_texture(&format!("config.materials.{name}.texture"), texture_storage)?;
                Ok(Self::DiffuseLight(texture))
            }
            "DIELECTRIC" => {
                let value = require_value(
                    table,
                    "refractive_index",
                    &format!("config.materials.{name}"),
                )?;
                let refractive_index =
                    value.parse_floatlike(&format!("config.materials.{name}.refractive_index"))?;

                Ok(Self::Dielectric { refractive_index })
            }
            "ISOTROPIC" => {
                let value = require_value(table, "texture", &format!("config.materials.{name}"))?;
                let texture = value
                    .parse_texture(&format!("config.materials.{name}.texture"), texture_storage)?;
                Ok(Self::Isotropic(texture))
            }
            "SOLIDCOLOR" => {
                // shortcut for a Lambertian material with an anonymous SolidColor texture
                let value = require_value(table, "color", &format!("config.materials.{name}"))?;
                let color = value.parse_color(&format!("config.materials.{name}.color"))?;
                let tex_id = texture_storage.push_anon(TextureModel::SolidColor { color });
                Ok(Self::Lambertian(tex_id))
            }
            "COLOREDLIGHT" => {
                // shortcut for a DiffuseLight material with an anonymous SolidColor texture
                let value = require_value(table, "color", &format!("config.materials.{name}"))?;
                let mut color = value.parse_color(&format!("config.materials.{name}.color"))?;

                if let Some(bright) = table.get("brightness") {
                    let brightness =
                        bright.parse_floatlike(&format!("config.materials.{name}.brightness"))?;
                    color.set_brightness(brightness);
                }

                let tex_id = texture_storage.push_anon(TextureModel::SolidColor { color });
                Ok(Self::DiffuseLight(tex_id))
            }
            _ => {
                bail!(miette::diagnostic!(
                    help = format!(
                        "valid material types include: {}",
                        r#""metal" | "light" | "lambertian" | "dielectric""#.purple()
                    ),
                    "{} must be a valid material type.",
                    format!("config.materials.{}.type", name).green(),
                ));
            }
        }
    }
}

impl ObjectModel {
    pub fn parse(
        index: usize,
        table: &toml::Table,
        materials: &MaterialStorage,
        objects: &mut Vec<Self>,
    ) -> Result<Self> {
        let Some(toml::Value::String(obj_type)) = table.get("type") else {
            bail!(
                "{} must be a string.",
                format!("config.materials.{}.type", index).green()
            );
        };

        match &obj_type.to_ascii_uppercase()[..] {
            "SPHERE" => {
                let value = require_value(table, "center", &format!("config.objects.{index}"))?;
                let center = value.parse_point3(&format!("config.objects.{index}.center"))?;
                let value = require_value(table, "radius", &format!("config.objects.{index}"))?;
                let radius = value.parse_floatlike(&format!("config.objects.{index}.radius"))?;
                let value = require_value(table, "material", &format!("config.objects.{index}"))?;
                let material =
                    value.parse_material(&format!("config.objects.{index}.material"), materials)?;
                Ok(Self::Sphere {
                    center,
                    radius,
                    material,
                })
            }
            _ => {
                bail!(miette::diagnostic!(
                    help = format!(
                        "valid object types include: {}",
                        r#""sphere" | "parallelogram" | "triangle" | "disc""#.purple()
                    ),
                    "{} must be a valid object type.",
                    format!("config.objects.{}.type", index).green(),
                ));
            }
        }
    }
}

impl ConfigModel {
    pub fn from_table(table: &toml::Table) -> Result<Self> {
        let Some(toml::Value::Table(texture_table)) = table.get("textures") else {
            bail!("{} must be a table.", "config.textures".green());
        };

        let Some(toml::Value::Table(material_table)) = table.get("materials") else {
            bail!("{} must be a table.", "config.materials".green());
        };

        let Some(toml::Value::Array(object_array)) = table.get("objects") else {
            bail!("{} must be a list of tables.", "config.objects".green());
        };

        let mut textures = TextureStorage::with_capacity(texture_table.len());
        let mut materials = HashMap::with_capacity(texture_table.len());
        let mut objects = Vec::with_capacity(object_array.len());

        for (texture_id, texture) in texture_table {
            let toml::Value::Table(texture_table) = texture else {
                bail!(
                    "{} must be a table.",
                    format!("config.textures.{}", texture_id).green()
                );
            };

            let texture = TextureModel::parse(texture_id, texture_table, &mut textures)?;
            textures.push_named(texture_id.clone(), texture);
        }

        for (material_id, material) in material_table {
            let toml::Value::Table(material_table) = material else {
                bail!(
                    "{} must be a table.",
                    format!("config.materials.{}", material_id).green()
                );
            };

            materials.insert(
                material_id.clone(),
                MaterialModel::parse(material_id, material_table, &mut textures)?,
            );
        }

        for (i, object) in object_array.iter().enumerate() {
            let toml::Value::Table(object_table) = object else {
                bail!(
                    "{} must be a table.",
                    format!("config.objects.{}", i).green()
                );
            };

            ObjectModel::parse(i, object_table, &materials, &mut objects)?;
        }

        Ok(Self {
            textures,
            materials,
            objects,
        })
    }
}

#[derive(Debug)]
struct World(HittableVec);

impl World {
    pub fn from_file_contents(s: &str) -> Result<Self, miette::Error> {
        let conf: ConfigModel = s.parse()?;
        let world = HittableVec::new();
        Ok(Self(world))
    }
}

impl FromStr for World {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_file_contents(s)
    }
}

impl std::ops::Deref for World {
    type Target = HittableVec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for ConfigModel {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_table(&s.parse::<toml::Table>().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const SAMPLE: &'static str = r##"

[textures.red]
type = "SolidColor"
color = "#ff0000"

[textures.cx]
type = "Checkerboard"
textures = ["#ff0", 0xfff]
scale = 1.0

[textures.world]
type = "Image"
path = "assets/textures/earth.png"

[materials.world]
type = "Lambertian"
texture = "world"

[materials.solid_red]
# Shortcut for Lambertian material with SolidColor texture
type = "SolidColor"
color = 0xff0000

[materials.metal]
type = "Lambertian"
albedo = 0xFFD700
fuzz = 0.1

[materials.light]
type = "Light"
texture = "world"

[materials.light2]
type = "ColoredLight"
color = 0xfff
# 10x white
brightness = 10

[[objects]]
type = "Parallelogram"
origin = [-3, -2, 5]
u = [0, 0, -4]
v = [0, 4, 0]
material = "left_red"
"##;

    #[test]
    fn deser() -> Result<()> {
        let _cfg: ConfigModel = SAMPLE.parse()?;
        Ok(())
    }
}
