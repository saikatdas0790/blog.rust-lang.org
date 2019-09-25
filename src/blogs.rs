use crate::posts::Post;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::path::{Path, PathBuf};

static MANIFEST_FILE: &str = "blog.yml";
static POSTS_EXT: &str = "md";

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct Manifest {
    pub(crate) title: String,
    pub(crate) index_title: String,
    pub(crate) description: String,
    pub(crate) maintained_by: String,
    pub(crate) requires_team: bool,
}

#[derive(Serialize)]
pub(crate) struct Blog {
    title: String,
    index_title: String,
    description: String,
    maintained_by: String,
    #[serde(serialize_with = "add_postfix_slash")]
    prefix: PathBuf,
    posts: Vec<Post>,
}

impl Blog {
    fn load(prefix: PathBuf, dir: &Path) -> Result<Self, Box<dyn Error>> {
        let manifest_content = std::fs::read_to_string(dir.join(MANIFEST_FILE))?;
        let manifest: Manifest = serde_yaml::from_str(&manifest_content)?;

        let mut posts = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if path.metadata()?.file_type().is_file() && ext == Some(POSTS_EXT) {
                posts.push(Post::open(&path, &manifest)?);
            }
        }

        posts.sort_by_key(|post| post.url.clone());
        posts.reverse();

        // Decide which posts should show the year in the index.
        posts[0].show_year = true;
        for i in 1..posts.len() {
            posts[i].show_year = posts[i - 1].year != posts[i].year;
        }

        Ok(Blog {
            title: manifest.title,
            index_title: manifest.index_title,
            description: manifest.description,
            maintained_by: manifest.maintained_by,
            prefix,
            posts,
        })
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn index_title(&self) -> &str {
        &self.index_title
    }

    pub(crate) fn prefix(&self) -> &Path {
        &self.prefix
    }

    pub(crate) fn posts(&self) -> &[Post] {
        &self.posts
    }
}

/// Recursively load blogs in a directory. A blog is a directory with a `blog.yml`
/// file inside it.
pub(crate) fn load(base: &Path) -> Result<Vec<Blog>, Box<dyn Error>> {
    let mut blogs = Vec::new();
    load_recursive(base, base, &mut blogs)?;
    Ok(blogs)
}

fn load_recursive(
    base: &Path,
    current: &Path,
    blogs: &mut Vec<Blog>,
) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(current)? {
        let path = entry?.path();
        let file_type = path.metadata()?.file_type();

        if file_type.is_dir() {
            load_recursive(base, &path, blogs)?;
        } else if file_type.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str());
            if let (Some(file_name), Some(parent)) = (file_name, path.parent()) {
                if file_name == MANIFEST_FILE {
                    let prefix = parent
                        .strip_prefix(base)
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|_| PathBuf::new());
                    blogs.push(Blog::load(prefix, parent)?);
                }
            }
        }
    }
    Ok(())
}

fn add_postfix_slash<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut str_repr = path.to_string_lossy().to_string();
    if !str_repr.is_empty() {
        str_repr.push('/');
    }
    serializer.serialize_str(&str_repr)
}
