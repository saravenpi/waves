use crate::app::WavesApp;
use crate::types::{FileEntry, Liked, SidebarView};

impl WavesApp {
    pub fn toggle_favorite(&mut self) {
        let entry = match self.sidebar_view {
            SidebarView::FileBrowser => {
                if self.columns.is_empty() || self.columns[0].entries.is_empty() {
                    return;
                }
                self.columns[0].entries.get(self.columns[0].selected).cloned()
            }
            SidebarView::Liked => {
                if self.liked.is_empty() || self.liked_selected >= self.liked.len() {
                    return;
                }
                let fav = &self.liked[self.liked_selected];
                Some(FileEntry {
                    name: fav.name.clone(),
                    path: fav.path.clone(),
                    is_dir: fav.is_dir,
                })
            }
            SidebarView::Settings => {
                return;
            }
        };

        if let Some(entry) = entry {
            if entry.is_dir {
                return;
            }

            if let Some(pos) = self.liked.iter().position(|f| f.path == entry.path) {
                self.liked.remove(pos);
                if self.liked_selected >= self.liked.len() && self.liked_selected > 0 {
                    self.liked_selected = self.liked.len() - 1;
                }
            } else {
                self.liked.insert(0, Liked {
                    path: entry.path.clone(),
                    name: entry.name.clone(),
                    is_dir: entry.is_dir,
                    timestamp: std::time::SystemTime::now(),
                });
                self.liked_selected = 0;
            }
            crate::liked::save(&self.liked);
        }
    }
}
