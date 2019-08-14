//! This module extends GadgetBuilder with a method for verifying permutations.

use std::collections::HashMap;

use crate::bimap_util::bimap_from_lists;
use crate::expression::{BooleanExpression, Expression};
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire::{BooleanWire, Wire};
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// Assert that two lists of expressions evaluate to permutations of one another.
    ///
    /// This is currently implemented using an AS-Waksman permutation network, although that could
    /// change in the future. See "On Arbitrary Waksman Networks and their Vulnerability".
    pub fn assert_permutation(&mut self, a: &[Expression<F>], b: &[Expression<F>]) {
        assert_eq!(a.len(), b.len(), "Permutation must have same number of inputs and outputs");
        let n = a.len();

        match n {
            // Two empty lists are permutations of one another, trivially.
            0 => return,
            // Two singleton lists are permutations of one another as long as their items are equal.
            1 => self.assert_equal(&a[0], &b[0]),
            // For the 2x2 case, we're implementing a switch gadget. The switch will be controlled
            // with a binary wire. It will swap swap the order of its inputs iff that wire is true.
            2 => self.assert_permutation_2x2(&a[0], &a[1], &b[0], &b[1]),
            // For larger lists, we recursively use two smaller permutation networks.
            _ => self.assert_permutation_recursive(a, b)
        }
    }

    /// Assert that [a, b] is a permutation of [c, d].
    fn assert_permutation_2x2(&mut self,
                              a: &Expression<F>, b: &Expression<F>,
                              c: &Expression<F>, d: &Expression<F>) {
        let (switch, c_target, d_target) = self.create_switch(a, b);
        self.assert_equal(c, c_target);
        self.assert_equal(d, d_target);
        let a = a.clone();
        let b = b.clone();
        let c = c.clone();
        let d = d.clone();
        self.generator(
            [a.dependencies(), b.dependencies(), c.dependencies(), d.dependencies()].concat(),
            move |values: &mut WireValues<F>| {
                let a_value = a.evaluate(values);
                let b_value = b.evaluate(values);
                let c_value = c.evaluate(values);
                let d_value = d.evaluate(values);
                if a_value == c_value && b_value == d_value {
                    values.set_boolean(switch, false);
                } else if a_value == d_value && b_value == c_value {
                    values.set_boolean(switch, true);
                } else {
                    panic!("No permutation from [{}, {}] to [{}, {}]",
                           a_value, b_value, c_value, d_value);
                }
            });
    }

    /// Creates a 2x2 switch given the two input expressions. Returns three things: the (boolean)
    /// switch wire and the two output expressions. The order of the outputs will match that of the
    /// inputs if the switch wire is set to false, otherwise the order will be swapped.
    fn create_switch(&mut self, a: &Expression<F>, b: &Expression<F>)
                     -> (BooleanWire, Expression<F>, Expression<F>) {
        let switch = self.boolean_wire();
        let c = self.selection(BooleanExpression::from(switch), b, a);
        let d = a + b - &c;
        (switch, c, d)
    }

    fn assert_permutation_recursive(&mut self, a: &[Expression<F>], b: &[Expression<F>]) {
        let n = a.len();
        let even = n % 2 == 0;

        let mut child_1_a = Vec::new();
        let mut child_1_b = Vec::new();
        let mut child_2_a = Vec::new();
        let mut child_2_b = Vec::new();

        // See Figure 8 in the AS-Waksman paper.
        let a_num_switches = n / 2;
        let b_num_switches = if even { a_num_switches - 1 } else { a_num_switches };

        let mut a_switches = Vec::new();
        let mut b_switches = Vec::new();
        for i in 0..a_num_switches {
            let (switch, out_1, out_2) = self.create_switch(&a[i * 2], &a[i * 2 + 1]);
            a_switches.push(switch);
            child_1_a.push(out_1);
            child_2_a.push(out_2);
        }
        for i in 0..b_num_switches {
            let (switch, out_1, out_2) = self.create_switch(&b[i * 2], &b[i * 2 + 1]);
            b_switches.push(switch);
            child_1_b.push(out_1);
            child_2_b.push(out_2);
        }

        // See Figure 8 in the AS-Waksman paper.
        if even {
            child_1_b.push(b[n - 2].clone());
            child_2_b.push(b[n - 1].clone());
        } else {
            child_2_a.push(a[n - 1].clone());
            child_2_b.push(b[n - 1].clone());
        }

        self.assert_permutation(&child_1_a, &child_1_b);
        self.assert_permutation(&child_2_a, &child_2_b);

        let a_deps: Vec<Wire> = a.iter().flat_map(Expression::dependencies).collect();
        let b_deps: Vec<Wire> = b.iter().flat_map(Expression::dependencies).collect();

        let a = a.to_vec();
        let b = b.to_vec();
        self.generator(
            [a_deps, b_deps].concat(),
            move |values: &mut WireValues<F>| {
                let a_values: Vec<Element<F>> = a.iter().map(|e| e.evaluate(values)).collect();
                let b_values: Vec<Element<F>> = b.iter().map(|e| e.evaluate(values)).collect();
                route(a_values, b_values, &a_switches, &b_switches, values);
            });
    }
}

/// Generates switch settings for a single layer of the recursive network.
fn route<F: Field>(a_values: Vec<Element<F>>, b_values: Vec<Element<F>>,
                   a_switches: &[BooleanWire], b_switches: &[BooleanWire],
                   values: &mut WireValues<F>) {
    assert_eq!(a_values.len(), b_values.len());
    let n = a_values.len();
    let even = n % 2 == 0;
    let ab_map = bimap_from_lists(a_values, b_values);
    let switches = [a_switches, b_switches];

    let ab_map_by_side = |side: usize, index: usize| -> usize {
        *match side {
            0 => ab_map.get_by_left(&index),
            1 => ab_map.get_by_right(&index),
            _ => panic!("Expected side to be 0 or 1")
        }.unwrap()
    };

    // We maintain two maps for wires which have been routed to a particular subnetwork on one side
    // of the network (left or right) but not the other. The keys are wire indices, and the values
    // are subnetwork indices.
    let mut partial_routes = [HashMap::new(), HashMap::new()];

    // After we route a wire on one side, we find the corresponding wire on the other side and check
    // if it still needs to be routed. If so, we add it to partial_routes.
    let enqueue_other_side = |partial_routes: &mut [HashMap<usize, bool>],
                              values: &mut WireValues<F>,
                              side: usize, this_i: usize, subnet: bool| {
        let other_side = 1 - side;
        let other_i = ab_map_by_side(side, this_i);
        let other_switch_i = other_i / 2;

        if other_switch_i >= switches[other_side].len() {
            // The other wire doesn't go through a switch, so there's no routing to be done.
            return;
        }

        if values.contains_boolean(switches[other_side][other_switch_i]) {
            // The other switch has already been routed.
            return;
        }

        let other_i_sibling = 4 * other_switch_i + 1 - other_i;
        if let Some(&sibling_subnet) = partial_routes[other_side].get(&other_i_sibling) {
            // The other switch's sibling is already pending routing.
            assert_ne!(subnet, sibling_subnet);
            return;
        }

        let opt_old_subnet = partial_routes[other_side].insert(other_i, subnet);
        if let Some(old_subnet) = opt_old_subnet {
            assert_eq!(subnet, old_subnet, "Routing conflict (should never happen)");
        }
    };

    // See Figure 8 in the AS-Waksman paper.
    if even {
        enqueue_other_side(&mut partial_routes, values, 1, n - 2, false);
        enqueue_other_side(&mut partial_routes, values, 1, n - 1, true);
    } else {
        enqueue_other_side(&mut partial_routes, values, 0, n - 1, true);
        enqueue_other_side(&mut partial_routes, values, 1, n - 1, true);
    }

    let route_switch = |partial_routes: &mut [HashMap<usize, bool>], values: &mut WireValues<F>,
                        side: usize, switch_index: usize, swap: bool| {
        // First, we actually set the switch configuration.
        values.set_boolean(switches[side][switch_index], swap);

        // Then, we enqueue the two corresponding wires on the other side of the network, to ensure
        // that they get routed in the next step.
        let this_i_1 = switch_index * 2;
        let this_i_2 = this_i_1 + 1;
        enqueue_other_side(partial_routes, values, side, this_i_1, swap);
        enqueue_other_side(partial_routes, values, side, this_i_2, !swap);
    };

    // If {a,b}_only_routes is empty, then we can route any switch next. For efficiency, we will
    // simply do top-down scans (one on the left side, one on the right side) for switches which
    // have not yet been routed. These variables represent the positions of those two scans.
    let mut scan_index = [0, 0];

    // Until both scans complete, we alternate back and worth between the left and right switch
    // layers. We process any partially routed wires for that side, or if there aren't any, we route
    // the next switch in our scan.
    while scan_index[0] < switches[0].len() || scan_index[1] < switches[1].len() {
        for side in 0..=1 {
            if !partial_routes[side].is_empty() {
                for (this_i, subnet) in partial_routes[side].clone().into_iter() {
                    let this_first_switch_input = this_i % 2 == 0;
                    let swap = this_first_switch_input == subnet;
                    let this_switch_i = this_i / 2;
                    route_switch(&mut partial_routes, values, side, this_switch_i, swap);
                }
                partial_routes[side].clear();
            } else {
                // We can route any switch next. Continue our scan for pending switches.
                while scan_index[side] < switches[side].len()
                    && values.contains_boolean(switches[side][scan_index[side]]) {
                    scan_index[side] += 1;
                }
                if scan_index[side] < switches[side].len() {
                    // Either switch configuration would work; we arbitrarily choose to not swap.
                    route_switch(&mut partial_routes, values, side, scan_index[side], false);
                    scan_index[side] += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;
    use crate::wire_values::WireValues;

    #[test]
    fn route_2x2() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        builder.assert_permutation(
            &[1u8.into(), 2u8.into()],
            &[2u8.into(), 1u8.into()]);
        let gadget = builder.build();
        assert!(gadget.execute(&mut WireValues::new()));
    }

    #[test]
    fn route_3x3() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        builder.assert_permutation(
            &[1u8.into(), 2u8.into(), 3u8.into()],
            &[2u8.into(), 1u8.into(), 3u8.into()]);
        let gadget = builder.build();
        assert!(gadget.execute(&mut WireValues::new()));
    }

    #[test]
    fn route_5x5() {
        type F = Bn128;
        let mut builder = GadgetBuilder::<F>::new();
        let a = builder.wires(5);
        let b = builder.wires(5);
        let a_exp: Vec<Expression<F>> = a.iter().map(Expression::from).collect();
        let b_exp: Vec<Expression<F>> = b.iter().map(Expression::from).collect();
        builder.assert_permutation(&a_exp, &b_exp);
        let gadget = builder.build();

        let mut values_normal = values!(
            a[0] => 0u8.into(), a[1] => 1u8.into(), a[2] => 2u8.into(), a[3] => 3u8.into(), a[4] => 4u8.into(),
            b[0] => 1u8.into(), b[1] => 4u8.into(), b[2] => 0u8.into(), b[3] => 3u8.into(), b[4] => 2u8.into());
        assert!(gadget.execute(&mut values_normal));

        let mut values_with_duplicates = values!(
            a[0] => 0u8.into(), a[1] => 1u8.into(), a[2] => 2u8.into(), a[3] => 0u8.into(), a[4] => 1u8.into(),
            b[0] => 1u8.into(), b[1] => 1u8.into(), b[2] => 0u8.into(), b[3] => 0u8.into(), b[4] => 2u8.into());
        assert!(gadget.execute(&mut values_with_duplicates));
    }

    #[test]
    #[should_panic]
    fn not_a_permutation() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        builder.assert_permutation(
            &[1u8.into(), 2u8.into(), 2u8.into()],
            &[1u8.into(), 2u8.into(), 1u8.into()]);
        let gadget = builder.build();
        // The generator should fail, since there's no possible routing.
        gadget.execute(&mut WireValues::new());
    }

    #[test]
    #[should_panic]
    fn lengths_differ() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        builder.assert_permutation(
            &[1u8.into(), 2u8.into(), 3u8.into()],
            &[1u8.into(), 2u8.into()]);
    }
}