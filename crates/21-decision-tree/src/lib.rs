// ============================================================
//  YOUR CHALLENGE — implement a CART decision tree classifier.
//
//  A decision tree recursively partitions the feature space.
//  At each node, find the feature f and threshold t that best
//  separates the classes — measured by information gain:
//
//    Gain = Gini(parent) - weighted_avg(Gini(left), Gini(right))
//
//  Gini impurity:
//    Gini(S) = 1 - Σ p_i²   (p_i = fraction of class i)
//    Pure node  → 0.0
//    50/50 split → 0.5
//
//  Used in: credit scoring, fraud detection, clinical triage,
//           churn prediction, scikit-learn's RandomForest.
// ============================================================

/// A node in the decision tree.
#[derive(Debug)]
pub enum Node {
    Leaf  { prediction: bool },
    Split { feature: usize, threshold: f64, left: Box<Node>, right: Box<Node> },
}

pub struct DecisionTree {
    root: Option<Node>,
    max_depth: usize,
}

impl DecisionTree {
    pub fn new(max_depth: usize) -> Self {
        Self { root: None, max_depth }
    }

    /// Fit the tree to labelled data.  `x[i]` is the feature vector for sample i.
    pub fn fit(&mut self, x: &[Vec<f64>], y: &[bool]) {
        self.root = Some(build_node(x, y, 0, self.max_depth));
    }

    /// Predict the label for a single sample.
    pub fn predict(&self, sample: &[f64]) -> bool {
        predict_node(self.root.as_ref().expect("call fit() first"), sample)
    }

    /// Predict labels for many samples.
    pub fn predict_many(&self, samples: &[Vec<f64>]) -> Vec<bool> {
        samples.iter().map(|s| self.predict(s)).collect()
    }
}

/// Gini impurity: probability that a randomly chosen label is wrong.
pub fn gini_impurity(labels: &[bool]) -> f64 {
    if labels.is_empty() { return 0.0; }
    let n = labels.len() as f64;
    let pos = labels.iter().filter(|&&l| l).count() as f64;
    let neg = n - pos;
    1.0 - (pos / n).powi(2) - (neg / n).powi(2)
}

/// Weighted Gini reduction from splitting parent into left/right.
pub fn information_gain(parent: &[bool], left: &[bool], right: &[bool]) -> f64 {
    let n = parent.len() as f64;
    gini_impurity(parent)
        - (left.len() as f64 / n)  * gini_impurity(left)
        - (right.len() as f64 / n) * gini_impurity(right)
}

// ── private helpers ────────────────────────────────────────────────────────

fn majority_class(y: &[bool]) -> bool {
    y.iter().filter(|&&l| l).count() * 2 >= y.len()
}

fn build_node(x: &[Vec<f64>], y: &[bool], depth: usize, max_depth: usize) -> Node {
    // Base cases: pure node, single sample, or depth limit
    if y.is_empty() || gini_impurity(y) < 1e-10 || depth >= max_depth {
        return Node::Leaf { prediction: majority_class(y) };
    }

    if let Some((feature, threshold)) = best_split(x, y) {
        let (left_x, left_y, right_x, right_y) = split_data(x, y, feature, threshold);
        if left_y.is_empty() || right_y.is_empty() {
            return Node::Leaf { prediction: majority_class(y) };
        }
        Node::Split {
            feature,
            threshold,
            left:  Box::new(build_node(&left_x,  &left_y,  depth + 1, max_depth)),
            right: Box::new(build_node(&right_x, &right_y, depth + 1, max_depth)),
        }
    } else {
        Node::Leaf { prediction: majority_class(y) }
    }
}

fn predict_node(node: &Node, sample: &[f64]) -> bool {
    match node {
        Node::Leaf  { prediction }              => *prediction,
        Node::Split { feature, threshold, left, right } =>
            if sample[*feature] <= *threshold { predict_node(left,  sample) }
            else                              { predict_node(right, sample) },
    }
}

/// Find the (feature, threshold) pair that maximises information gain.
fn best_split(x: &[Vec<f64>], y: &[bool]) -> Option<(usize, f64)> {
    let n_features = x[0].len();
    let mut best_gain = 0.0_f64;
    let mut best = None;

    for feature in 0..n_features {
        let mut vals: Vec<f64> = x.iter().map(|xi| xi[feature]).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vals.dedup();

        for i in 0..vals.len().saturating_sub(1) {
            let threshold = (vals[i] + vals[i + 1]) / 2.0;
            let (_, ly, _, ry) = split_data(x, y, feature, threshold);
            if ly.is_empty() || ry.is_empty() { continue; }
            let gain = information_gain(y, &ly, &ry);
            if gain > best_gain {
                best_gain = gain;
                best = Some((feature, threshold));
            }
        }
    }
    best
}

fn split_data(x: &[Vec<f64>], y: &[bool], feature: usize, threshold: f64)
    -> (Vec<Vec<f64>>, Vec<bool>, Vec<Vec<f64>>, Vec<bool>)
{
    let mut lx = Vec::new(); let mut ly = Vec::new();
    let mut rx = Vec::new(); let mut ry = Vec::new();
    for (xi, &yi) in x.iter().zip(y.iter()) {
        if xi[feature] <= threshold { lx.push(xi.clone()); ly.push(yi); }
        else                        { rx.push(xi.clone()); ry.push(yi); }
    }
    (lx, ly, rx, ry)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod impurity {
        use super::*;

        #[test]
        fn gini_pure_node_is_zero() {
            assert!((gini_impurity(&[true, true, true])).abs() < 1e-10);
            assert!((gini_impurity(&[false, false])).abs() < 1e-10);
        }

        #[test]
        fn gini_balanced_split_is_half() {
            let labels = vec![true, false, true, false];
            assert!((gini_impurity(&labels) - 0.5).abs() < 1e-10);
        }

        #[test]
        fn information_gain_is_positive_for_perfect_split() {
            let parent = vec![true, true, false, false];
            let left   = vec![true, true];
            let right  = vec![false, false];
            let gain   = information_gain(&parent, &left, &right);
            assert!(gain > 0.0, "perfect split should have positive gain: {gain}");
        }

        #[test]
        fn information_gain_is_zero_for_useless_split() {
            let parent = vec![true, false, true, false];
            let left   = vec![true, false];
            let right  = vec![true, false];
            let gain   = information_gain(&parent, &left, &right);
            assert!(gain.abs() < 1e-10, "useless split should have ~0 gain: {gain}");
        }
    }

    mod tree {
        use super::*;

        #[test]
        fn linearly_separable_data_achieves_perfect_accuracy() {
            // Class true when x > 0.5
            let x: Vec<Vec<f64>> = (0..10).map(|i| vec![i as f64 / 10.0]).collect();
            let y: Vec<bool>     = x.iter().map(|xi| xi[0] > 0.5).collect();
            let mut tree = DecisionTree::new(3);
            tree.fit(&x, &y);
            let preds = tree.predict_many(&x);
            assert_eq!(preds, y, "should classify linearly separable data perfectly");
        }

        #[test]
        fn all_same_class_predicts_that_class() {
            let x = vec![vec![0.0], vec![1.0], vec![2.0]];
            let y = vec![true, true, true];
            let mut tree = DecisionTree::new(3);
            tree.fit(&x, &y);
            assert_eq!(tree.predict(&[1.5]), true);
        }

        #[test]
        fn max_depth_1_creates_single_split() {
            let x: Vec<Vec<f64>> = (0..20).map(|i| vec![i as f64]).collect();
            let y: Vec<bool>     = x.iter().map(|xi| xi[0] >= 10.0).collect();
            let mut tree = DecisionTree::new(1);
            tree.fit(&x, &y);
            // With depth=1 it should still get the main split right
            assert_eq!(tree.predict(&[5.0]),  false);
            assert_eq!(tree.predict(&[15.0]), true);
        }

        #[test]
        fn predict_many_is_consistent_with_predict() {
            let x: Vec<Vec<f64>> = (0..8).map(|i| vec![i as f64 * 0.25]).collect();
            let y: Vec<bool>     = x.iter().map(|xi| xi[0] > 0.5).collect();
            let mut tree = DecisionTree::new(4);
            tree.fit(&x, &y);
            let batch  = tree.predict_many(&x);
            let single: Vec<bool> = x.iter().map(|xi| tree.predict(xi)).collect();
            assert_eq!(batch, single);
        }
    }
}
