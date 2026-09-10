// function createSignal(initial) {
//     let value = initial

//     function getter() {
//         trackDependency()
//         return value
//     }
//     function setter(newValue) {
//         value = newValue
//         notifySubscribers()
//     }

//     return [getter, setter]
// }
//

// #[interface]
// struct User{
//     name: String,
//     age:u8,
// }

// #[component]
// fn userButton(){
//     let (user, SetUser) = signal(User{"Nigga", 18});
//     view!{
//         <div className="border border-white">
//             {user()}
//         </div>
//     }
// }

// #[component]
// fn personButton(){
//     let (user, SetUser) = signal(Person{"Nigga", 18});
//     view!{
//         <div className="border border-white">
//             {user()}
//         </div>
//     }
// }

// #[component]
// fn Root(){
//     view!{
//         <personButton user={}/>
//         <userButton /->
//     }
// }

use gpui::{App, AppContext, Entity};

pub fn create_signal<T: 'static + Clone>(
    initial: T,
    cx: &mut App,
) -> (impl Fn(&App) -> T + Clone, impl Fn(T, &mut App) + Clone) {
    let entity: Entity<T> = cx.new(|_cx| initial);

    let getter = {
        let entity = entity.clone();
        move |cx: &App| entity.read(cx).clone()
    };

    let setter = move |value: T, cx: &mut App| {
        entity.update(cx, |state, cx| {
            *state = value;
            cx.notify();
        });
    };

    (getter, setter)
}
