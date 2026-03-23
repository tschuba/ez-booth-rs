use domain::models::booth::Booth;
use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedBoothContext(pub RwSignal<Option<Booth>>);

pub fn provide_selected_booth_context() -> RwSignal<Option<Booth>> {
    let booth_signal = create_rw_signal(None::<Booth>);
    provide_context(SelectedBoothContext(booth_signal));
    booth_signal
}

pub fn use_selected_booth() -> RwSignal<Option<Booth>> {
    use_context::<SelectedBoothContext>()
        .expect("SelectedBoothContext not found. Did you call provide_selected_booth_context() at the root?")
        .0
}

#[component]
pub fn SelectedBoothProvider(children: Children) -> impl IntoView {
    provide_selected_booth_context();
    children()
}
