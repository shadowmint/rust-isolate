use crate::IsolateIdentity;
use crate::isolate_runtime::IsolateRef;

pub struct IsolateRuntimeSharedState<T: Clone + Send + 'static, TState: Default + Send + 'static> {
    refs: HashMap<IsolateIdentity, IsolateRef<T>>,
    state: TState,
}

impl<T: Clone + Send + 'static, TState: Default + Send + 'static> IsolateRuntimeSharedState<T, TState> {
    pub fn new() -> Arc<Mutex<IsolateRuntimeSharedState<T, TState>>> {
        Arc::new(Mutex::new(IsolateRuntimeSharedState {
            refs: HashMap::new(),
            state: Default::default(),
        }))
    }
}
