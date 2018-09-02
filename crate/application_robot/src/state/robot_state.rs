use std::cell::RefCell;
use std::rc::Rc;

use amethyst::prelude::*;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

use state::intercept::ApplicationEventIntercept;
use state::Intercept;

/// Wraps a delegate state with automation capabilities.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RobotState<T, E>
where
    T: 'static,
    E: Send + Sync + 'static,
{
    /// Intercepts to track and control application behaviour.
    ///
    /// Rc<RefCell<Intercept<T, E>>> is a trait object, which does not implement Sized, needed by the generated
    /// setter from the `Builder` derive, so we instead provide default intercepts, and functions
    /// to toggle the enablement of certain `Intercept`s.
    #[derivative(Debug = "ignore")]
    pub intercepts: Vec<Rc<RefCell<Intercept<T, E>>>>,
    /// State to delegate behaviour to.
    #[derivative(Debug = "ignore")]
    pub delegate: Box<State<T, E>>,
}

impl<T, E> RobotState<T, E>
where
    T: 'static,
    E: Send + Sync + 'static,
{
    /// Returns a new application robot state.
    pub fn new(delegate: Box<State<T, E>>) -> Self {
        RobotState {
            intercepts: RobotState::default_intercepts(),
            delegate,
        } // kcov-ignore
    }

    /// Returns a new application robot state with only the specified intercepts.
    pub fn new_with_intercepts(
        delegate: Box<State<T, E>>,
        intercepts: Vec<Rc<RefCell<Intercept<T, E>>>>,
    ) -> Self {
        RobotState {
            intercepts,
            delegate,
        } // kcov-ignore
    }

    /// Returns the default intercepts for a `RobotState`.
    ///
    /// Currently this only includes the `ApplicationEventIntercept`.
    pub fn default_intercepts() -> Vec<Rc<RefCell<Intercept<T, E>>>> {
        vec![Rc::new(RefCell::new(ApplicationEventIntercept::new()))]
    }

    fn fold_trans_begin<F>(&mut self, mut intercept_fn: F) -> Option<Trans<T, E>>
    where
        F: FnMut(&mut Rc<RefCell<Intercept<T, E>>>) -> Option<Trans<T, E>>,
    {
        let trans_opt = self
            .intercepts
            .iter_mut()
            .fold_while(None, |trans, intercept| {
                if trans.is_none() {
                    Continue(intercept_fn(intercept))
                } else {
                    Done(trans)
                }
            }).into_inner();

        trans_opt.map(|trans| self.wrap_trans(trans))
    }

    fn fold_trans_end<F>(&mut self, state_trans: Trans<T, E>, mut intercept_fn: F) -> Trans<T, E>
    where
        F: FnMut(&mut Rc<RefCell<Intercept<T, E>>>, &Trans<T, E>) -> Option<Trans<T, E>>,
    {
        let intercept_trans = {
            let state_trans_ref = &state_trans;
            self.intercepts
                .iter_mut()
                .fold_while(None, |trans, intercept| {
                    if trans.is_none() {
                        Continue(intercept_fn(intercept, state_trans_ref))
                    } else {
                        Done(trans)
                    }
                }).into_inner()
        };
        self.wrap_trans(intercept_trans.unwrap_or(state_trans))
    }

    /// When returning a `Trans` with a `State`, wrap it with a `RobotState` with the transitive
    /// intercepts.
    fn wrap_trans(&mut self, trans: Trans<T, E>) -> Trans<T, E> {
        match trans {
            Trans::Push(state) => Trans::Push(self.wrap_trans_state(state)),
            Trans::Switch(state) => Trans::Switch(self.wrap_trans_state(state)),
            _ => trans,
        }
    }

    /// Returns the provided `trans_state` with a `RobotState` that shares this state's transitive
    /// `Intercept`s.
    ///
    /// # Parameters
    ///
    /// * `trans_state`: `State` that should be wrapped in a `RobotState`.
    fn wrap_trans_state(&mut self, trans_state: Box<State<T, E>>) -> Box<State<T, E>> {
        let intercepts = self
            .intercepts
            .iter()
            .filter(|intercept| intercept.borrow().is_transitive())
            .cloned()
            .collect::<Vec<Rc<RefCell<Intercept<T, E>>>>>();
        Box::new(RobotState {
            intercepts,
            delegate: trans_state,
        })
    }
}

impl<'a, 'b, T, E> State<T, E> for RobotState<T, E>
where
    T: 'static,
    E: Send + Sync + 'static,
{
    fn on_start(&mut self, mut data: StateData<T>) {
        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_start_begin(&mut data));

        self.delegate.on_start(data);

        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_start_end());
    }

    fn on_stop(&mut self, mut data: StateData<T>) {
        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_stop_begin(&mut data));

        self.delegate.on_stop(data);

        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_stop_end());
    }

    fn on_pause(&mut self, mut data: StateData<T>) {
        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_pause_begin(&mut data));

        self.delegate.on_pause(data);

        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_pause_end());
    }

    fn on_resume(&mut self, mut data: StateData<T>) {
        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_resume_begin(&mut data));

        self.delegate.on_resume(data);

        self.intercepts
            .iter_mut()
            .for_each(|intercept| intercept.borrow_mut().on_resume_end());
    }

    // TODO: Pending <https://gitlab.com/azriel91/autexousious/issues/16>
    // kcov-ignore-start
    fn handle_event(&mut self, mut data: StateData<T>, mut event: StateEvent<E>) -> Trans<T, E> {
        let intercept_trans = self.fold_trans_begin(|intercept| {
            intercept
                .borrow_mut()
                .handle_event_begin(&mut data, &mut event)
        });
        if let Some(trans) = intercept_trans {
            return trans;
        }

        let trans = self.delegate.handle_event(data, event);

        self.fold_trans_end(trans, |intercept, trans| {
            intercept.borrow_mut().handle_event_end(trans)
        })
    }
    // kcov-ignore-end

    fn fixed_update(&mut self, mut data: StateData<T>) -> Trans<T, E> {
        let intercept_trans =
            self.fold_trans_begin(|intercept| intercept.borrow_mut().fixed_update_begin(&mut data));
        if let Some(trans) = intercept_trans {
            return trans;
        }

        let trans = self.delegate.fixed_update(data);

        self.fold_trans_end(trans, |intercept, trans| {
            intercept.borrow_mut().fixed_update_end(trans)
        }) // kcov-ignore
    }

    fn update(&mut self, mut data: StateData<T>) -> Trans<T, E> {
        let intercept_trans =
            self.fold_trans_begin(|intercept| intercept.borrow_mut().update_begin(&mut data));
        if let Some(trans) = intercept_trans {
            return trans;
        }

        let trans = self.delegate.update(data);

        self.fold_trans_end(trans, |intercept, trans| {
            intercept.borrow_mut().update_end(trans)
        }) // kcov-ignore
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::fmt::{self, Debug};
    use std::rc::Rc;

    use amethyst::ecs::prelude::World;
    use amethyst::prelude::*;
    use amethyst::renderer::{Event, WindowEvent};
    use debug_util_amethyst::{assert_eq_trans, display_trans};
    use enigo::{Enigo, Key, KeyboardControllable};
    use winit::{ControlFlow, EventsLoop, Window};

    use super::RobotState;
    use state::Intercept;

    type Invocations = Rc<RefCell<Vec<Invocation>>>;

    fn setup<T, E>(
        invocations: Invocations,
        intercepts: Vec<Rc<RefCell<Intercept<T, E>>>>,
    ) -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let robot_state = RobotState::new_with_intercepts(
            Box::new(MockState::new(invocations.clone(), Trans::None)),
            intercepts,
        );

        let world = World::new();

        (robot_state, world, invocations)
    }

    fn setup_without_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        setup(Rc::new(RefCell::new(vec![])), Vec::new())
    }

    fn setup_with_no_op_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 1,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_begin_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 1,
                invocations: invocations.clone(),
                trans_begin: Some(Trans::Pop),
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 2,
                invocations: invocations.clone(),
                trans_begin: Some(Trans::Quit),
                trans_end: None,
                transitive: false,
            })),
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_push_begin_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 3,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: true,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 4,
                invocations: invocations.clone(),
                trans_begin: Some(Trans::Push(Box::new(MockState::new(
                    invocations.clone(),
                    Trans::None,
                )))),
                trans_end: None,
                transitive: false,
            })), // kcov-ignore
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_push_end_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 3,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: true,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 4,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: Some(Trans::Push(Box::new(MockState::new(
                    invocations.clone(),
                    Trans::None,
                )))),
                transitive: false,
            })), // kcov-ignore
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_switch_begin_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 3,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: true,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 5,
                invocations: invocations.clone(),
                trans_begin: Some(Trans::Switch(Box::new(MockState::new(
                    invocations.clone(),
                    Trans::None,
                )))),
                trans_end: None,
                transitive: false,
            })), // kcov-ignore
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_switch_end_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 3,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: true,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 5,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: Some(Trans::Switch(Box::new(MockState::new(
                    invocations.clone(),
                    Trans::None,
                )))),
                transitive: false,
            })), // kcov-ignore
        ];
        setup(invocations, intercepts)
    }

    fn setup_with_end_intercepts<T, E>() -> (RobotState<T, E>, World, Invocations)
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        let invocations = Rc::new(RefCell::new(vec![]));
        let intercepts: Vec<Rc<RefCell<Intercept<T, E>>>> = vec![
            Rc::new(RefCell::new(MockIntercept {
                id: 0,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: None,
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 1,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: Some(Trans::Pop),
                transitive: false,
            })),
            Rc::new(RefCell::new(MockIntercept {
                id: 2,
                invocations: invocations.clone(),
                trans_begin: None,
                trans_end: Some(Trans::Quit),
                transitive: false,
            })),
        ];
        setup(invocations, intercepts)
    }

    #[macro_use]
    macro_rules! delegate_test {
        ($test_name:ident, $function:ident, $invocation:expr) => {
            #[test]
            fn $test_name() {
                let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
                    setup_without_intercepts();

                state.$function(StateData::new(&mut world, &mut ()));

                assert_eq!(vec![$invocation], *invocations.borrow());
            }
        };
    }

    #[macro_use]
    macro_rules! intercept_no_op_test {
        ($test_name:ident, $function:ident, $($invocation:expr),* $(,)*) => {
            #[test]
            fn $test_name() {
                let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
                    setup_with_no_op_intercepts();

                state.$function(StateData::new(&mut world, &mut ()));

                assert_eq!(
                    vec![$($invocation,)*],
                    *invocations.borrow()
                );
            }
        }
    }

    #[macro_use]
    macro_rules! intercept_no_op_trans_test {
        ($test_name:ident, $function:ident, $expected_trans:expr, $($invocation:expr),* $(,)*) => {
            #[test]
            fn $test_name() {
                let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
                setup_with_no_op_intercepts();

                let trans = state.$function(StateData::new(&mut world, &mut ()));

                assert_eq_trans(&$expected_trans, &trans);
                assert_eq!(
                    vec![$($invocation,)*],
                    *invocations.borrow()
                );
            }
        }
    }

    #[macro_use]
    macro_rules! intercept_begin_test {
        ($test_name:ident, $function:ident, $expected_trans:expr, $($invocation:expr),* $(,)*) => {
            #[test]
            fn $test_name() {
                let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
                    setup_with_begin_intercepts();

                let trans = state.$function(StateData::new(&mut world, &mut ()));

                assert_eq_trans(&$expected_trans, &trans);
                assert_eq!(
                    vec![$($invocation,)*],
                    *invocations.borrow()
                );
            }
        }
    }

    #[macro_use]
    macro_rules! intercept_end_test {
        ($test_name:ident, $function:ident, $expected_trans:expr, $($invocation:expr),* $(,)*) => {
            #[test]
            fn $test_name() {
                let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
                    setup_with_end_intercepts();

                let trans = state.$function(StateData::new(&mut world, &mut ()));

                assert_eq_trans(&$expected_trans, &trans);
                assert_eq!(
                    vec![$($invocation,)*],
                    *invocations.borrow()
                );
            }
        }
    }

    delegate_test!(on_start_delegates_to_state, on_start, Invocation::OnStart);
    delegate_test!(on_stop_delegates_to_state, on_stop, Invocation::OnStop);
    delegate_test!(on_pause_delegates_to_state, on_pause, Invocation::OnPause);
    delegate_test!(
        on_resume_delegates_to_state,
        on_resume,
        Invocation::OnResume
    );
    delegate_test!(
        fixed_update_delegates_to_state,
        fixed_update,
        Invocation::FixedUpdate
    );
    delegate_test!(update_delegates_to_state, update, Invocation::Update);

    // TODO: We ignore running the following tests because we cannot construct a window in both
    // this test and in the application_event_intercept module due to
    // <https://gitlab.com/azriel91/autexousious/issues/16>.
    // kcov-ignore-start
    #[test]
    #[ignore]
    fn handle_event_delegates_to_state() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_without_intercepts();

        let mut events_loop = EventsLoop::new();
        let _window = Window::new(&events_loop).unwrap();
        let event = get_window_event(&mut events_loop);

        let trans = state.handle_event(
            StateData::new(&mut world, &mut ()),
            StateEvent::Window(event),
        );

        assert_eq_trans(&Trans::None, &trans);
        assert_eq!(vec![Invocation::HandleEvent], *invocations.borrow());
    }

    #[test]
    #[ignore]
    fn handle_event_invokes_intercept() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_no_op_intercepts();

        let mut events_loop = EventsLoop::new();
        let _window = Window::new(&events_loop).unwrap();
        let event = get_window_event(&mut events_loop);

        let trans = state.handle_event(
            StateData::new(&mut world, &mut ()),
            StateEvent::Window(event),
        );

        assert_eq_trans(&Trans::None, &trans);
        assert_eq!(
            vec![
                Invocation::HandleEventBegin(0),
                Invocation::HandleEventBegin(1),
                Invocation::HandleEvent,
                Invocation::HandleEventEnd(0),
                Invocation::HandleEventEnd(1),
            ],
            *invocations.borrow()
        );
    }

    #[test]
    #[ignore]
    fn handle_event_returns_intercept_trans_begin() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_begin_intercepts();

        let mut events_loop = EventsLoop::new();
        let _window = Window::new(&events_loop).unwrap();
        let event = get_window_event(&mut events_loop);

        let trans = state.handle_event(
            StateData::new(&mut world, &mut ()),
            StateEvent::Window(event),
        );

        assert_eq_trans(&Trans::Pop, &trans);
        assert_eq!(
            vec![
                Invocation::HandleEventBegin(0),
                Invocation::HandleEventBegin(1),
            ],
            *invocations.borrow()
        );
    }

    #[test]
    #[ignore]
    fn handle_event_returns_intercept_trans_end() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_end_intercepts();

        let mut events_loop = EventsLoop::new();
        let _window = Window::new(&events_loop).unwrap();
        let event = get_window_event(&mut events_loop);

        let trans = state.handle_event(
            StateData::new(&mut world, &mut ()),
            StateEvent::Window(event),
        );

        assert_eq_trans(&Trans::Pop, &trans);
        assert_eq!(
            vec![
                Invocation::HandleEventBegin(0),
                Invocation::HandleEventBegin(1),
                Invocation::HandleEventBegin(2),
                Invocation::HandleEvent,
                Invocation::HandleEventEnd(0),
                Invocation::HandleEventEnd(1),
            ],
            *invocations.borrow()
        );
    }
    // kcov-ignore-end

    intercept_no_op_test!(
        on_start_invokes_intercept,
        on_start,
        Invocation::OnStartBegin(0),
        Invocation::OnStartBegin(1),
        Invocation::OnStart,
        Invocation::OnStartEnd(0),
        Invocation::OnStartEnd(1),
    );
    intercept_no_op_test!(
        on_stop_invokes_intercept,
        on_stop,
        Invocation::OnStopBegin(0),
        Invocation::OnStopBegin(1),
        Invocation::OnStop,
        Invocation::OnStopEnd(0),
        Invocation::OnStopEnd(1),
    );
    intercept_no_op_test!(
        on_pause_invokes_intercept,
        on_pause,
        Invocation::OnPauseBegin(0),
        Invocation::OnPauseBegin(1),
        Invocation::OnPause,
        Invocation::OnPauseEnd(0),
        Invocation::OnPauseEnd(1),
    );
    intercept_no_op_test!(
        on_resume_invokes_intercept,
        on_resume,
        Invocation::OnResumeBegin(0),
        Invocation::OnResumeBegin(1),
        Invocation::OnResume,
        Invocation::OnResumeEnd(0),
        Invocation::OnResumeEnd(1),
    );
    intercept_no_op_trans_test!(
        fixed_update_invokes_intercept,
        fixed_update,
        Trans::None,
        Invocation::FixedUpdateBegin(0),
        Invocation::FixedUpdateBegin(1),
        Invocation::FixedUpdate,
        Invocation::FixedUpdateEnd(0),
        Invocation::FixedUpdateEnd(1),
    );
    intercept_no_op_trans_test!(
        update_invokes_intercept,
        update,
        Trans::None,
        Invocation::UpdateBegin(0),
        Invocation::UpdateBegin(1),
        Invocation::Update,
        Invocation::UpdateEnd(0),
        Invocation::UpdateEnd(1),
    );

    intercept_begin_test!(
        fixed_update_returns_intercept_trans_begin,
        fixed_update,
        Trans::Pop,
        Invocation::FixedUpdateBegin(0),
        Invocation::FixedUpdateBegin(1),
    );

    intercept_begin_test!(
        update_returns_intercept_trans_begin,
        update,
        Trans::Pop,
        Invocation::UpdateBegin(0),
        Invocation::UpdateBegin(1),
    );

    intercept_end_test!(
        fixed_update_returns_intercept_trans_end,
        fixed_update,
        Trans::Pop,
        Invocation::FixedUpdateBegin(0),
        Invocation::FixedUpdateBegin(1),
        Invocation::FixedUpdateBegin(2),
        Invocation::FixedUpdate,
        Invocation::FixedUpdateEnd(0),
        Invocation::FixedUpdateEnd(1),
    );

    intercept_end_test!(
        update_returns_intercept_trans_end,
        update,
        Trans::Pop,
        Invocation::UpdateBegin(0),
        Invocation::UpdateBegin(1),
        Invocation::UpdateBegin(2),
        Invocation::Update,
        Invocation::UpdateEnd(0),
        Invocation::UpdateEnd(1),
    );

    #[test]
    fn intercept_begin_push_state_is_wrapped_with_robot_state_with_transitive_intercepts() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_push_begin_intercepts();

        let mut trans = state.update(StateData::new(&mut world, &mut ()));

        let dummy_state = MockState::new(Rc::new(RefCell::new(vec![])), Trans::None);
        let expected_trans = Trans::Push(Box::new(dummy_state));
        assert_eq_trans(&expected_trans, &trans);

        if let Trans::Push(ref mut pushed_state) = trans {
            pushed_state.update(StateData::new(&mut world, &mut ()));
        }

        assert_eq!(
            vec![
                Invocation::UpdateBegin(0),
                Invocation::UpdateBegin(3),
                Invocation::UpdateBegin(4),
                // Push
                Invocation::UpdateBegin(3),
                Invocation::Update,
                Invocation::UpdateEnd(3),
            ],
            *invocations.borrow()
        );
    }

    #[test]
    fn intercept_end_push_state_is_wrapped_with_robot_state_with_transitive_intercepts() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_push_end_intercepts();

        let mut trans = state.update(StateData::new(&mut world, &mut ()));

        let dummy_state = MockState::new(Rc::new(RefCell::new(vec![])), Trans::None);
        let expected_trans = Trans::Push(Box::new(dummy_state));
        assert_eq_trans(&expected_trans, &trans);

        if let Trans::Push(ref mut pushed_state) = trans {
            pushed_state.update(StateData::new(&mut world, &mut ()));
        }

        assert_eq!(
            vec![
                Invocation::UpdateBegin(0),
                Invocation::UpdateBegin(3),
                Invocation::UpdateBegin(4),
                Invocation::Update,
                Invocation::UpdateEnd(0),
                Invocation::UpdateEnd(3),
                Invocation::UpdateEnd(4),
                // Push
                Invocation::UpdateBegin(3),
                Invocation::Update,
                Invocation::UpdateEnd(3),
            ],
            *invocations.borrow()
        );
    }

    #[test]
    fn intercept_begin_switch_state_is_wrapped_with_robot_state_with_transitive_intercepts() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_switch_begin_intercepts();

        let mut trans = state.update(StateData::new(&mut world, &mut ()));

        let dummy_state = MockState::new(Rc::new(RefCell::new(vec![])), Trans::None);
        let expected_trans = Trans::Switch(Box::new(dummy_state));
        assert_eq_trans(&expected_trans, &trans);

        if let Trans::Switch(ref mut pushed_state) = trans {
            pushed_state.update(StateData::new(&mut world, &mut ()));
        }

        assert_eq!(
            vec![
                Invocation::UpdateBegin(0),
                Invocation::UpdateBegin(3),
                Invocation::UpdateBegin(5),
                // Switch
                Invocation::UpdateBegin(3),
                Invocation::Update,
                Invocation::UpdateEnd(3),
            ],
            *invocations.borrow()
        );
    }

    #[test]
    fn intercept_end_switch_state_is_wrapped_with_robot_state_with_transitive_intercepts() {
        let (mut state, mut world, invocations): (RobotState<(), ()>, _, _) =
            setup_with_switch_end_intercepts();

        let mut trans = state.update(StateData::new(&mut world, &mut ()));

        let dummy_state = MockState::new(Rc::new(RefCell::new(vec![])), Trans::None);
        let expected_trans = Trans::Switch(Box::new(dummy_state));
        assert_eq_trans(&expected_trans, &trans);

        if let Trans::Switch(ref mut pushed_state) = trans {
            pushed_state.update(StateData::new(&mut world, &mut ()));
        }

        assert_eq!(
            vec![
                Invocation::UpdateBegin(0),
                Invocation::UpdateBegin(3),
                Invocation::UpdateBegin(5),
                Invocation::Update,
                Invocation::UpdateEnd(0),
                Invocation::UpdateEnd(3),
                Invocation::UpdateEnd(5),
                // Switch
                Invocation::UpdateBegin(3),
                Invocation::Update,
                Invocation::UpdateEnd(3),
            ],
            *invocations.borrow()
        );
    }

    #[derive(Debug, PartialEq)]
    enum Invocation {
        OnStart,
        OnStop,
        OnPause,
        OnResume,
        HandleEvent,
        FixedUpdate,
        Update,

        // `Intercept` invocations
        OnStartBegin(u32),
        OnStartEnd(u32),
        OnStopBegin(u32),
        OnStopEnd(u32),
        OnPauseBegin(u32),
        OnPauseEnd(u32),
        OnResumeBegin(u32),
        OnResumeEnd(u32),
        HandleEventBegin(u32), // kcov-ignore
        HandleEventEnd(u32),   // kcov-ignore
        FixedUpdateBegin(u32),
        FixedUpdateEnd(u32),
        UpdateBegin(u32),
        UpdateEnd(u32),
    }

    /// Declares a function that pushes the specified invocation to the `self.invocations` field.
    #[macro_use]
    macro_rules! fn_ {
        ($function:ident, $invocation:expr) => {
            fn $function(&mut self, _: StateData<T>) {
                self.invocations
                    .borrow_mut()
                    .push($invocation); // kcov-ignore
            }
        }
    }

    /// Declares a function that pushes the specified invocation to the `self.invocations` field.
    ///
    /// This macro passes the `self.id` field as a parameter to the `Invocation` variant.
    #[macro_use]
    macro_rules! fn_id {
        ($function:ident, $invocation:expr; [$($additional_param:ty),*]) => {
            fn $function(&mut self, $(_: $additional_param),*) {
                self.invocations
                    .borrow_mut()
                    .push($invocation(self.id));
            }
        }
    }

    /// Declares a function that pushes the specified invocation to the `self.invocations` field.
    ///
    /// The function returns the value in the `self.trans` field, which is expected to contain a
    /// value.
    #[macro_use]
    macro_rules! fn_trans {
        ($function:ident, $invocation:expr; [$($additional_param:ty),*]) => {
            fn $function(&mut self, $(_: $additional_param),*) -> Trans<T, E> {
                self.invocations
                    .borrow_mut()
                    .push($invocation); // kcov-ignore

                self.trans.take().unwrap()
            }
        }
    }

    /// Declares a function that pushes the specified invocation to the `self.invocations` field.
    ///
    /// The function returns the optional value in the `self.$trans` field
    #[macro_use]
    macro_rules! fn_opt_trans {
        ($function:ident, $invocation:expr, $trans:ident; [$($additional_param:ty),*]) => {
            fn $function(&mut self, $(_: $additional_param),*) -> Option<Trans<T, E>> {
                self.invocations
                    .borrow_mut()
                    .push($invocation(self.id));

                self.$trans.take()
            }
        }
    }

    struct MockIntercept<T, E>
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        id: u32,
        invocations: Invocations,
        trans_begin: Option<Trans<T, E>>,
        trans_end: Option<Trans<T, E>>,
        transitive: bool,
    }

    impl<T, E> Intercept<T, E> for MockIntercept<T, E>
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        fn_id!(on_start_begin, Invocation::OnStartBegin; [&mut StateData<T>]);
        fn_id!(on_start_end, Invocation::OnStartEnd; []);
        fn_id!(on_stop_begin, Invocation::OnStopBegin; [&mut StateData<T>]);
        fn_id!(on_stop_end, Invocation::OnStopEnd; []);
        fn_id!(on_pause_begin, Invocation::OnPauseBegin; [&mut StateData<T>]);
        fn_id!(on_pause_end, Invocation::OnPauseEnd; []);
        fn_id!(on_resume_begin, Invocation::OnResumeBegin; [&mut StateData<T>]);
        fn_id!(on_resume_end, Invocation::OnResumeEnd; []);
        fn_opt_trans!(
            handle_event_begin,
            Invocation::HandleEventBegin,
            trans_begin;
            [&mut StateData<T>, &mut StateEvent<E>]
        );
        fn_opt_trans!(handle_event_end, Invocation::HandleEventEnd, trans_end; [&Trans<T, E>]);
        fn_opt_trans!(fixed_update_begin, Invocation::FixedUpdateBegin, trans_begin; [&mut StateData<T>]);
        fn_opt_trans!(fixed_update_end, Invocation::FixedUpdateEnd, trans_end; [&Trans<T, E>]);
        fn_opt_trans!(update_begin, Invocation::UpdateBegin, trans_begin; [&mut StateData<T>]);
        fn_opt_trans!(update_end, Invocation::UpdateEnd, trans_end; [&Trans<T, E>]);
        fn is_transitive(&self) -> bool {
            self.transitive
        }
    }

    // kcov-ignore-start
    impl<T, E> Debug for MockIntercept<T, E>
    where
        E: Send + Sync + 'static,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(
                f,
                "MockIntercept {{ invocations: {:?}, trans_begin: {}, trans_end: {} }}",
                self.invocations,
                format_trans(&self.trans_begin),
                format_trans(&self.trans_end),
            )
        }
    }
    // kcov-ignore-end

    #[derive(Default)]
    struct MockState<T, E>
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        invocations: Invocations,
        trans: Option<Trans<T, E>>,
    }

    impl<T, E> MockState<T, E>
    where
        E: Send + Sync + 'static,
    {
        fn new(invocations: Invocations, trans: Trans<T, E>) -> Self {
            MockState {
                invocations,
                trans: Some(trans),
            }
        }
    }

    impl<T, E> Debug for MockState<T, E>
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(
                f,
                "MockState {{ invocations: {:?}, trans: {} }}",
                self.invocations,
                format_trans(&self.trans),
            )
        }
    }

    impl<'a, 'b, T, E> State<T, E> for MockState<T, E>
    where
        T: 'static,
        E: Send + Sync + 'static,
    {
        fn_!(on_start, Invocation::OnStart);
        fn_!(on_stop, Invocation::OnStop);
        fn_!(on_pause, Invocation::OnPause);
        fn_!(on_resume, Invocation::OnResume);
        fn_trans!(
            handle_event,
            Invocation::HandleEvent;
            [StateData<T>, StateEvent<E>]
        );
        fn_trans!(fixed_update, Invocation::FixedUpdate; [StateData<T>]);
        fn_trans!(update, Invocation::Update; [StateData<T>]);
    }

    // TODO: Pending <https://gitlab.com/azriel91/autexousious/issues/16>
    // kcov-ignore-start
    fn get_window_event(events_loop: &mut EventsLoop) -> Event {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::Backspace);

        let mut return_event = None;

        events_loop.run_forever(|event| {
            if match &event {
                &Event::WindowEvent {
                    event: ref window_event,
                    ..
                } => match window_event {
                    &WindowEvent::KeyboardInput { .. } => true,
                    _ => false,
                },
                _ => false,
            } {
                return_event = Some(event);
                ControlFlow::Break
            } else {
                ControlFlow::Continue
            }
        });

        events_loop.poll_events(|_event| {}); // empty event queue

        return_event.unwrap()
    }

    fn format_trans<T, E>(trans: &Option<Trans<T, E>>) -> String {
        if trans.is_some() {
            format!("Some({})", display_trans(trans.as_ref().unwrap()))
        } else {
            "None".to_string()
        }
    }
    // kcov-ignore-end
}
