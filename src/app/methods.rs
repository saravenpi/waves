use crate::app::WavesApp;
use crate::types::Column;
use crate::file_operations::browser::read_directory;

impl WavesApp {
    /// Updates the file browser columns using the current selection.
    pub fn update_columns(&mut self) {
        self.update_columns_with_selection(None);
    }

    /// Updates the file browser columns with a specified selection index.
    ///
    /// # Arguments
    /// * `selection` - Optional index to select in the updated column
    pub fn update_columns_with_selection(&mut self, selection: Option<usize>) {
        let current_selection = if let Some(sel) = selection {
            sel
        } else if !self.columns.is_empty() {
            self.columns[0].selected
        } else {
            0
        };

        self.columns.clear();

        let current_entries = read_directory(&self.current_dir);

        let selected = if current_entries.is_empty() {
            0
        } else {
            current_selection.min(current_entries.len().saturating_sub(1))
        };

        let current_column = Column {
            entries: current_entries,
            selected,
        };
        self.columns.push(current_column);
    }
}
