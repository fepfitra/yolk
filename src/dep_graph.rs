use std::collections::{HashMap, HashSet};

/// Error types for dependency resolution
#[derive(Debug, Clone)]
pub enum DepError {
    /// Circular dependency detected: module depends on itself or creates a cycle
    CircularDependency { module: String, path: Vec<String> },
    /// Referenced dependency does not exist
    MissingDependency { module: String, missing: String },
}

impl std::fmt::Display for DepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepError::CircularDependency { module, path } => {
                write!(
                    f,
                    "Circular dependency detected for '{}': {}",
                    module,
                    path.join(" -> ")
                )
            }
            DepError::MissingDependency { module, missing } => {
                write!(
                    f,
                    "Module '{}' depends on non-existent module '{}'",
                    module, missing
                )
            }
        }
    }
}

impl std::error::Error for DepError {}

/// Node in the dependency graph
#[derive(Debug, Clone)]
pub struct DepNode {
    pub name: String,
    pub depends_on: Vec<String>,
    pub dependents: Vec<String>, // Reverse edges: who depends on this
    pub enabled: bool,
}

impl DepNode {
    pub fn new(name: String, depends_on: Vec<String>, enabled: bool) -> Self {
        Self {
            name,
            depends_on,
            dependents: Vec::new(),
            enabled,
        }
    }
}

/// Dependency graph for eggs
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    nodes: HashMap<String, DepNode>,
}

impl DepGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, name: String, depends_on: Vec<String>, enabled: bool) {
        // Use and_modify to update existing nodes, or insert new ones
        self.nodes
            .entry(name.clone())
            .and_modify(|node| {
                node.depends_on = depends_on.clone();
                node.enabled = enabled;
            })
            .or_insert_with(|| DepNode::new(name.clone(), depends_on.clone(), enabled));

        // Also ensure dependency nodes exist (but don't overwrite their deps)
        for dep in depends_on {
            self.nodes
                .entry(dep.clone())
                .or_insert_with(|| DepNode::new(dep, Vec::new(), true));
        }
    }

    /// Get a node by name
    pub fn get(&self, name: &str) -> Option<&DepNode> {
        self.nodes.get(name)
    }

    /// Get all node names
    pub fn node_names(&self) -> Vec<&String> {
        self.nodes.keys().collect()
    }

    /// Get all nodes
    pub fn nodes_iter(&self) -> impl Iterator<Item = &DepNode> {
        self.nodes.values()
    }

    /// Build reverse edges (who depends on me)
    pub fn build_reverse_edges(&mut self) {
        for node in self.nodes.values_mut() {
            node.dependents.clear();
        }
        let mut edges: Vec<(String, String)> = Vec::new();
        for (name, node) in self.nodes.iter() {
            for dep in &node.depends_on {
                edges.push((dep.clone(), name.clone()));
            }
        }
        for (dep, dependent) in edges {
            if let Some(dep_node) = self.nodes.get_mut(&dep) {
                dep_node.dependents.push(dependent);
            }
        }
    }

    /// Perform topological sort to determine execution order
    /// Returns nodes in order such that dependencies come before dependents
    pub fn topological_sort(&self) -> Result<Vec<&DepNode>, DepError> {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut in_progress: HashSet<&str> = HashSet::new();
        let mut result: Vec<&DepNode> = Vec::new();
        let mut path: Vec<String> = Vec::new();

        // Sort nodes by name for deterministic order
        let mut sorted_nodes: Vec<&str> = self.nodes.keys().map(|s| s.as_str()).collect();
        sorted_nodes.sort();

        for node_name in sorted_nodes {
            self.visit(
                node_name,
                &mut visited,
                &mut in_progress,
                &mut result,
                &mut path,
            )?;
        }

        Ok(result)
    }

    fn visit<'a>(
        &'a self,
        name: &'a str,
        visited: &mut HashSet<&'a str>,
        in_progress: &mut HashSet<&'a str>,
        result: &mut Vec<&'a DepNode>,
        path: &mut Vec<String>,
    ) -> Result<(), DepError> {
        if visited.contains(name) {
            return Ok(());
        }

        if in_progress.contains(name) {
            // Found a cycle - trace back through path
            path.push(name.to_string());
            return Err(DepError::CircularDependency {
                module: name.to_string(),
                path: path.clone(),
            });
        }

        in_progress.insert(name);
        path.push(name.to_string());

        if let Some(node) = self.nodes.get(name) {
            for dep in &node.depends_on {
                self.visit(dep, visited, in_progress, result, path)?;
            }
        }

        in_progress.remove(name);
        path.pop();
        visited.insert(name);

        if let Some(node) = self.nodes.get(name) {
            result.push(node);
        }

        Ok(())
    }

    /// Check for circular dependencies
    pub fn detect_cycles(&self) -> Result<(), DepError> {
        self.topological_sort()?;
        Ok(())
    }

    /// Validate that all dependencies exist
    pub fn validate_dependencies(&self) -> Result<(), DepError> {
        for node in self.nodes.values() {
            for dep in &node.depends_on {
                if !self.nodes.contains_key(dep) {
                    return Err(DepError::MissingDependency {
                        module: node.name.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_dependency() {
        let mut graph = DepGraph::new();
        graph.add_node("a".to_string(), vec![], true);
        graph.add_node("b".to_string(), vec!["a".to_string()], true);
        graph.add_node("c".to_string(), vec!["b".to_string()], true);
        graph.build_reverse_edges();

        let sorted = graph.topological_sort().unwrap();
        let names: Vec<&str> = sorted.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = DepGraph::new();
        graph.add_node("a".to_string(), vec![], true);
        graph.add_node("b".to_string(), vec!["a".to_string()], true);
        graph.add_node("c".to_string(), vec!["a".to_string()], true);
        graph.add_node(
            "d".to_string(),
            vec!["b".to_string(), "c".to_string()],
            true,
        );
        graph.build_reverse_edges();

        let sorted = graph.topological_sort().unwrap();
        let names: Vec<&str> = sorted.iter().map(|n| n.name.as_str()).collect();
        // d should come last, a should come first
        assert_eq!(names[0], "a");
        assert_eq!(names[3], "d");
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = DepGraph::new();
        graph.add_node("a".to_string(), vec!["b".to_string()], true);
        graph.add_node("b".to_string(), vec!["a".to_string()], true);
        graph.build_reverse_edges();

        let result = graph.topological_sort();
        assert!(result.is_err());
    }
}
