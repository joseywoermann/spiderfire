/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;

use mozjs::jsapi::{
	Handle, JS_GetConstructor, JS_GetInstancePrivate, JS_InitClass, JS_NewObjectWithGivenProto, JSClass, JSFunctionSpec, JSPropertySpec, SetPrivate,
};

use crate::{Arguments, Context, Function, NativeFunction, Object};

// TODO: Move into Context Wrapper
thread_local!(pub static CLASS_INFOS: RefCell<HashMap<TypeId, ClassInfo>> = RefCell::new(HashMap::new()));

#[derive(Copy, Clone, Debug)]
pub struct ClassInfo {
	#[allow(dead_code)]
	constructor: Function,
	prototype: Object,
}

pub trait ClassInitialiser {
	fn class() -> &'static JSClass;

	fn parent_info(_: Context) -> Option<ClassInfo> {
		None
	}

	fn constructor() -> (NativeFunction, u32);

	fn functions() -> &'static [JSFunctionSpec] {
		&[JSFunctionSpec::ZERO]
	}

	fn properties() -> &'static [JSPropertySpec] {
		&[JSPropertySpec::ZERO]
	}

	fn static_functions() -> &'static [JSFunctionSpec] {
		&[JSFunctionSpec::ZERO]
	}

	fn static_properties() -> &'static [JSPropertySpec] {
		&[JSPropertySpec::ZERO]
	}

	fn init_class(cx: Context, object: &Object) -> ClassInfo
	where
		Self: Sized + 'static,
	{
		let class = Self::class();
		let parent_proto = Self::parent_info(cx).map(|ci| ci.prototype).unwrap_or_else(|| Object::new(cx));
		let (constructor, nargs) = Self::constructor();
		let properties = Self::properties();
		let functions = Self::functions();
		let static_properties = Self::static_properties();
		let static_functions = Self::static_functions();

		rooted!(in(cx) let parent_prototype = *parent_proto);
		rooted!(in(cx) let object = **object);
		let class = unsafe {
			JS_InitClass(
				cx,
				object.handle().into(),
				parent_prototype.handle().into(),
				class,
				Some(constructor),
				nargs,
				properties.as_ptr() as *const _,
				functions.as_ptr() as *const _,
				static_properties.as_ptr() as *const _,
				static_functions.as_ptr() as *const _,
			)
		};

		rooted!(in(cx) let rclass = class);
		let constructor = unsafe { JS_GetConstructor(cx, rclass.handle().into()) };

		let class_info = ClassInfo {
			constructor: Function::from_object(constructor).unwrap(),
			prototype: Object::from(class),
		};

		CLASS_INFOS.with(|infos| {
			let mut infos = infos.borrow_mut();
			(*infos).insert(TypeId::of::<Self>(), class_info);
			class_info
		})
	}

	fn new_object(cx: Context, native: Self) -> Object
	where
		Self: Sized + 'static,
	{
		CLASS_INFOS.with(|infos| {
			let infos = infos.borrow();
			let info = (*infos).get(&TypeId::of::<Self>()).expect("Uninitialised Class");
			let b = Box::new(native);
			unsafe {
				let obj = JS_NewObjectWithGivenProto(cx, Self::class(), Handle::from_marked_location(&*info.prototype));
				SetPrivate(obj, Box::into_raw(b) as *mut c_void);
				Object::from(obj)
			}
		})
	}

	fn get_private<'a>(cx: Context, obj: Object, args: Option<&Arguments>) -> &'a mut Self
	where
		Self: Sized,
	{
		unsafe {
			rooted!(in(cx) let obj = *obj);
			let args = args.map(|a| a.call_args()).as_mut().map_or(ptr::null_mut(), |args| args);
			let ptr = JS_GetInstancePrivate(cx, obj.handle().into(), Self::class(), args) as *mut Self;
			&mut *ptr
		}
	}
}
