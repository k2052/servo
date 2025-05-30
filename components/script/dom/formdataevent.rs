/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding::FormDataEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::formdata::FormData;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct FormDataEvent {
    event: Event,
    form_data: Dom<FormData>,
}

impl FormDataEvent {
    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        form_data: &FormData,
        can_gc: CanGc,
    ) -> DomRoot<FormDataEvent> {
        Self::new_with_proto(
            window, None, type_, can_bubble, cancelable, form_data, can_gc,
        )
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        form_data: &FormData,
        can_gc: CanGc,
    ) -> DomRoot<FormDataEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(FormDataEvent {
                event: Event::new_inherited(),
                form_data: Dom::from_ref(form_data),
            }),
            window,
            proto,
            can_gc,
        );

        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(can_bubble), bool::from(cancelable));
        }
        ev
    }
}

impl FormDataEventMethods<crate::DomTypeHolder> for FormDataEvent {
    // https://html.spec.whatwg.org/multipage/#formdataevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &FormDataEventBinding::FormDataEventInit,
    ) -> Fallible<DomRoot<FormDataEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);

        let event = FormDataEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.formData.clone(),
            can_gc,
        );

        Ok(event)
    }

    // https://html.spec.whatwg.org/multipage/#dom-formdataevent-formdata
    fn FormData(&self) -> DomRoot<FormData> {
        DomRoot::from_ref(&*self.form_data)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
