use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{FunctionType, StructType},
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace, IntPredicate,
};

/* -----------------------------------------------------------------------

 Vector Backend Infraestructure

 Functions:

    - Comming soon...

----------------------------------------------------------------------- */

pub struct VectorAPI<'a, 'ctx> {
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    context: &'ctx Context,
    vector_type: StructType<'ctx>,
}

impl<'a, 'ctx> VectorAPI<'a, 'ctx> {
    pub fn include(module: &'a Module<'ctx>, builder: &'a Builder<'ctx>, context: &'ctx Context) {
        Self {
            module,
            builder,
            context,
            vector_type: context.struct_type(
                &[
                    context.i64_type().into(),                        // size
                    context.i64_type().into(),                        // capacity
                    context.i64_type().into(),                        // element_size
                    context.ptr_type(AddressSpace::default()).into(), // data
                    context.i8_type().into(),                         // type
                ],
                false,
            ),
        }
        .start_construction()
    }

    pub fn define(module: &'a Module<'ctx>, builder: &'a Builder<'ctx>, context: &'ctx Context) {
        Self {
            module,
            builder,
            context,
            vector_type: context.struct_type(
                &[
                    context.i64_type().into(),                        // size
                    context.i64_type().into(),                        // capacity
                    context.i64_type().into(),                        // element_size
                    context.ptr_type(AddressSpace::default()).into(), // data
                    context.i8_type().into(),                         // type
                ],
                false,
            ),
        }
        .start_definition()
    }

    fn start_construction(&mut self) {
        self.needed_functions();
        self.init();
        self.should_grow();
        self.size_at_bytes();
        self.realloc();
        self.adjust_capacity();
        self.size();
        self.data();
        self.push_i8();
        self.destroy();
    }

    fn start_definition(&mut self) {
        self.define_init();
        self.define_size();
        self.define_data();
        self.define_push_i8();
        self.define_destroy();
    }

    /*

        CONSTRUCTION FUNCTIONS (START)

    */

    fn init(&mut self) {
        let vector_init_type: FunctionType<'_> = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.i8_type().into(),
            ],
            true,
        );

        let vector_init: FunctionValue<'_> =
            self.module.add_function("Vec.init", vector_init_type, None);

        let vector_init_block: BasicBlock<'_> = self.context.append_basic_block(vector_init, "");

        self.builder.position_at_end(vector_init_block);

        let alloca_capacity: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.i64_type(), "")
            .unwrap();

        let alloca_element_size: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.i64_type(), "")
            .unwrap();

        self.builder
            .build_store(
                alloca_capacity,
                vector_init.get_nth_param(1).unwrap().into_int_value(),
            )
            .unwrap();

        self.builder
            .build_store(
                alloca_element_size,
                vector_init.get_nth_param(2).unwrap().into_int_value(),
            )
            .unwrap();

        let size: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                vector_init.get_first_param().unwrap().into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        self.builder
            .build_store(size, self.context.i64_type().const_zero())
            .unwrap();

        let capacity: IntValue<'ctx> = self
            .builder
            .build_load(self.context.i64_type(), alloca_capacity, "")
            .unwrap()
            .into_int_value();

        let idiomatic_capacity = self
            .builder
            .build_call(
                self.module.get_function("llvm.umax.i64").unwrap(),
                &[
                    self.context.i64_type().const_int(2, false).into(),
                    capacity.into(),
                ],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();

        let capacity: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                vector_init.get_first_param().unwrap().into_pointer_value(),
                1,
                "",
            )
            .unwrap();

        self.builder
            .build_store(capacity, idiomatic_capacity)
            .unwrap();

        let element_size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), alloca_element_size, "")
            .unwrap()
            .into_int_value();

        let element_size_2: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                vector_init.get_first_param().unwrap().into_pointer_value(),
                2,
                "",
            )
            .unwrap();

        self.builder
            .build_store(element_size_2, element_size)
            .unwrap();

        let capacity: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), alloca_capacity, "")
            .unwrap()
            .into_int_value();

        let element_size_3: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), alloca_element_size, "")
            .unwrap()
            .into_int_value();

        let malloc_size: IntValue<'ctx> = self
            .builder
            .build_int_mul(capacity, element_size_3, "")
            .unwrap();

        let data_allocated: PointerValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("malloc").unwrap(),
                &[malloc_size.into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        let data: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                vector_init.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        self.builder.build_store(data, data_allocated).unwrap();

        let get_type: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                vector_init.get_first_param().unwrap().into_pointer_value(),
                4,
                "",
            )
            .unwrap();

        self.builder
            .build_store(
                get_type,
                vector_init.get_last_param().unwrap().into_int_value(),
            )
            .unwrap();

        self.builder.build_return(None).unwrap();
    }

    fn should_grow(&mut self) {
        let should_grow: FunctionValue<'_> = self.module.add_function(
            "_Vec.should_grow",
            self.context.bool_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
            Some(Linkage::LinkerPrivate),
        );

        let should_grow_block: BasicBlock<'_> = self.context.append_basic_block(should_grow, "");

        self.builder.position_at_end(should_grow_block);

        let size_get: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                should_grow.get_first_param().unwrap().into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), size_get, "")
            .unwrap()
            .into_int_value();

        let capacity_get: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                should_grow.get_first_param().unwrap().into_pointer_value(),
                1,
                "",
            )
            .unwrap();

        let capacity: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), capacity_get, "")
            .unwrap()
            .into_int_value();

        let cmp: IntValue<'_> = self
            .builder
            .build_int_compare(IntPredicate::EQ, size, capacity, "")
            .unwrap();

        self.builder.build_return(Some(&cmp)).unwrap();
    }

    fn adjust_capacity(&mut self) {
        let adjust_capacity: FunctionValue<'_> = self.module.add_function(
            "_Vec.adjust_capacity",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
            Some(Linkage::LinkerPrivate),
        );

        let adjust_capacity_block: BasicBlock<'_> =
            self.context.append_basic_block(adjust_capacity, "");

        self.builder.position_at_end(adjust_capacity_block);

        let size_get: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                adjust_capacity
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), size_get, "")
            .unwrap()
            .into_int_value();

        let grow: IntValue<'ctx> = self
            .builder
            .build_int_mul(size, self.context.i64_type().const_int(2, false), "")
            .unwrap();

        let size_get_2: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                adjust_capacity
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let size_2: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), size_get_2, "")
            .unwrap()
            .into_int_value();

        let new_capacity: IntValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("llvm.umax.i64").unwrap(),
                &[size_2.into(), grow.into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        self.builder
            .build_call(
                self.module.get_function("Vec.realloc").unwrap(),
                &[
                    adjust_capacity
                        .get_first_param()
                        .unwrap()
                        .into_pointer_value()
                        .into(),
                    new_capacity.into(),
                ],
                "",
            )
            .unwrap();

        self.builder.build_return(None).unwrap();
    }

    fn size_at_bytes(&mut self) {
        let size_at_bytes: FunctionValue<'_> = self.module.add_function(
            "_Vec.size_at_bytes",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
            Some(Linkage::LinkerPrivate),
        );

        let size_at_bytes_block: BasicBlock<'_> =
            self.context.append_basic_block(size_at_bytes, "");

        self.builder.position_at_end(size_at_bytes_block);

        let size_get: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                size_at_bytes
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let element_size_get = self
            .builder
            .build_struct_gep(
                self.vector_type,
                size_at_bytes
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value(),
                2,
                "",
            )
            .unwrap();

        let size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), size_get, "")
            .unwrap()
            .into_int_value();

        let element_size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), element_size_get, "")
            .unwrap()
            .into_int_value();

        let size_in_bytes: IntValue<'_> =
            self.builder.build_int_mul(size, element_size, "").unwrap();

        self.builder.build_return(Some(&size_in_bytes)).unwrap();
    }

    fn realloc(&mut self) {
        let realloc: FunctionValue<'_> = self.module.add_function(
            "Vec.realloc",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                ],
                true,
            ),
            Some(Linkage::LinkerPrivate),
        );

        let realloc_block: BasicBlock<'_> = self.context.append_basic_block(realloc, "");

        self.builder.position_at_end(realloc_block);

        let alloca_new_capacity: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.i64_type(), "")
            .unwrap();

        self.builder
            .build_store(alloca_new_capacity, realloc.get_last_param().unwrap())
            .unwrap();

        let get_capacity: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                1,
                "",
            )
            .unwrap();

        let capacity: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), get_capacity, "")
            .unwrap()
            .into_int_value();

        let new_capacity: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), alloca_new_capacity, "")
            .unwrap()
            .into_int_value();

        let new_capacity_in_bytes: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.i64_type(), "")
            .unwrap();

        let old_data: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();

        let cmp_capacity: IntValue<'_> = self
            .builder
            .build_int_compare(
                IntPredicate::ULT,
                new_capacity,
                self.context.i64_type().const_int(2, false),
                "",
            )
            .unwrap();

        let is_capacity_true: BasicBlock<'_> = self.context.append_basic_block(realloc, "");
        let is_capacity_false: BasicBlock<'_> = self.context.append_basic_block(realloc, "");

        self.builder
            .build_conditional_branch(cmp_capacity, is_capacity_true, is_capacity_false)
            .unwrap();

        self.builder.position_at_end(is_capacity_true);

        let cmp_capacity_2: IntValue<'ctx> = self
            .builder
            .build_int_compare(
                IntPredicate::UGT,
                capacity,
                self.context.i64_type().const_int(2, false),
                "",
            )
            .unwrap();

        let is_capacity_true_2: BasicBlock<'_> = self.context.append_basic_block(realloc, "");
        let is_capacity_false_2: BasicBlock<'_> = self.context.append_basic_block(realloc, "");

        self.builder
            .build_conditional_branch(cmp_capacity_2, is_capacity_true_2, is_capacity_false_2)
            .unwrap();

        self.builder.position_at_end(is_capacity_true_2);

        self.builder
            .build_store(
                alloca_new_capacity,
                self.context.i64_type().const_int(2, false),
            )
            .unwrap();

        self.builder
            .build_unconditional_branch(is_capacity_false)
            .unwrap();

        self.builder.position_at_end(is_capacity_false_2);
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(is_capacity_false);

        let get_element_size = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                2,
                "",
            )
            .unwrap();

        let element_size = self
            .builder
            .build_load(self.context.i64_type(), get_element_size, "")
            .unwrap()
            .into_int_value();

        let get_new_capacity_2: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), alloca_new_capacity, "")
            .unwrap()
            .into_int_value();

        let new_capacity_in_bytes_to_allocate = self
            .builder
            .build_int_mul(get_new_capacity_2, element_size, "")
            .unwrap();

        self.builder
            .build_store(new_capacity_in_bytes, new_capacity_in_bytes_to_allocate)
            .unwrap();

        let get_data = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        self.builder.build_store(old_data, get_data).unwrap();

        let get_data_2 = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        let load_new_capacity_in_bytes: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), new_capacity_in_bytes, "")
            .unwrap()
            .into_int_value();

        let new_data = self
            .builder
            .build_call(
                self.module.get_function("malloc").unwrap(),
                &[load_new_capacity_in_bytes.into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        self.builder.build_store(get_data_2, new_data).unwrap();
        let get_data_3 = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        let get_data_4 = self
            .builder
            .build_load(get_data_3.get_type(), get_data_3, "")
            .unwrap()
            .into_pointer_value();

        let get_data_5 = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        let get_data_6 = self
            .builder
            .build_load(get_data_5.get_type(), get_data_5, "")
            .unwrap()
            .into_pointer_value();

        let size_at_bytes: IntValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("_Vec.size_at_bytes").unwrap(),
                &[get_data_4.into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let get_capacity = self
            .builder
            .build_struct_gep(
                self.vector_type,
                realloc.get_first_param().unwrap().into_pointer_value(),
                1,
                "",
            )
            .unwrap();

        self.builder
            .build_store(get_capacity, new_capacity)
            .unwrap();

        self.builder
            .build_call(
                self.module.get_function("llvm.memcpy.p0.p0.i64").unwrap(),
                &[
                    get_data_6.into(),
                    old_data.into(),
                    size_at_bytes.into(),
                    self.context.bool_type().const_zero().into(),
                ],
                "",
            )
            .unwrap();

        self.builder.build_return(None).unwrap();
    }

    fn push_i8(&mut self) {
        let push_i8: FunctionValue<'_> = self.module.add_function(
            "Vec.push_i8",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i8_type().into(),
                ],
                true,
            ),
            None,
        );

        let block_push_i8: BasicBlock<'_> = self.context.append_basic_block(push_i8, "");

        self.builder.position_at_end(block_push_i8);

        let should_grow: IntValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("_Vec.should_grow").unwrap(),
                &[push_i8
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value()
                    .into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let cmp: IntValue<'_> = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                should_grow,
                self.context.bool_type().const_int(1, false),
                "",
            )
            .unwrap();

        let then_block: BasicBlock<'_> = self.context.append_basic_block(push_i8, "");
        let else_block: BasicBlock<'_> = self.context.append_basic_block(push_i8, "");

        self.builder
            .build_conditional_branch(cmp, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);

        self.builder
            .build_call(
                self.module.get_function("_Vec.adjust_capacity").unwrap(),
                &[push_i8
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value()
                    .into()],
                "",
            )
            .unwrap();

        self.builder.build_unconditional_branch(else_block).unwrap();

        self.builder.position_at_end(else_block);

        let size: IntValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("Vec.size").unwrap(),
                &[push_i8
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value()
                    .into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let data: PointerValue<'_> = self
            .builder
            .build_call(
                self.module.get_function("Vec.data").unwrap(),
                &[push_i8
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value()
                    .into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        unsafe {
            let get_index = self
                .builder
                .build_in_bounds_gep(self.context.i8_type(), data, &[size], "")
                .unwrap();

            self.builder
                .build_store(get_index, push_i8.get_last_param().unwrap())
                .unwrap();
        }

        let new_size: IntValue<'_> = self
            .builder
            .build_int_add(size, self.context.i64_type().const_int(1, false), "")
            .unwrap();

        let get_size: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                push_i8.get_first_param().unwrap().into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        self.builder.build_store(get_size, new_size).unwrap();

        self.builder.build_return(None).unwrap();
    }

    fn data(&mut self) {
        let get_data: FunctionValue<'_> = self.module.add_function(
            "Vec.data",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        );

        let block_get_data: BasicBlock<'_> = self.context.append_basic_block(get_data, "");

        self.builder.position_at_end(block_get_data);

        let get_data: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                get_data.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        let data: PointerValue<'_> = self
            .builder
            .build_load(get_data.get_type(), get_data, "")
            .unwrap()
            .into_pointer_value();

        self.builder.build_return(Some(&data)).unwrap();
    }

    fn size(&mut self) {
        let get_size: FunctionValue<'_> = self.module.add_function(
            "Vec.size",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        );

        let block_get_size: BasicBlock<'_> = self.context.append_basic_block(get_size, "");

        self.builder.position_at_end(block_get_size);

        let get_size: PointerValue<'_> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                get_size.get_first_param().unwrap().into_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let size: IntValue<'_> = self
            .builder
            .build_load(self.context.i64_type(), get_size, "")
            .unwrap()
            .into_int_value();

        self.builder.build_return(Some(&size)).unwrap();
    }

    fn destroy(&mut self) {
        let destroy: FunctionValue<'_> = self.module.add_function(
            "Vec.destroy",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        );

        let block_destroy: BasicBlock<'_> = self.context.append_basic_block(destroy, "");

        self.builder.position_at_end(block_destroy);

        let get_data: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                destroy.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        let data: PointerValue<'_> = self
            .builder
            .build_load(get_data.get_type(), get_data, "")
            .unwrap()
            .into_pointer_value();

        self.builder.build_free(data).unwrap();

        let get_data_2: PointerValue<'ctx> = self
            .builder
            .build_struct_gep(
                self.vector_type,
                destroy.get_first_param().unwrap().into_pointer_value(),
                3,
                "",
            )
            .unwrap();

        self.builder
            .build_store(
                get_data_2,
                self.context.ptr_type(AddressSpace::default()).const_null(),
            )
            .unwrap();

        self.builder.build_return(None).unwrap();
    }

    fn needed_functions(&self) {
        self.module.add_function(
            "llvm.umax.i64",
            self.context.i64_type().fn_type(
                &[
                    self.context.i64_type().into(),
                    self.context.i64_type().into(),
                ],
                false,
            ),
            Some(Linkage::External),
        );

        self.module.add_function(
            "llvm.memcpy.p0.p0.i64",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.bool_type().into(),
                ],
                false,
            ),
            Some(Linkage::External),
        );

        self.module.add_function(
            "malloc",
            self.context
                .ptr_type(AddressSpace::default())
                .fn_type(&[self.context.i64_type().into()], false),
            Some(Linkage::External),
        );
    }

    /*

        CONSTRUCTION FUNCTIONS (END)

    */

    /*

        DEFINITION FUNCTIONS (START)

    */

    fn define_init(&mut self) {
        self.module.add_function(
            "Vec.init",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.i64_type().into(),
                    self.context.i8_type().into(),
                ],
                true,
            ),
            Some(Linkage::External),
        );
    }

    fn define_push_i8(&mut self) {
        self.module.add_function(
            "Vec.push_i8",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i8_type().into(),
                ],
                true,
            ),
            Some(Linkage::External),
        );
    }

    fn define_data(&mut self) {
        self.module.add_function(
            "Vec.data",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
    }

    fn define_size(&mut self) {
        self.module.add_function(
            "Vec.size",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
    }

    fn define_destroy(&mut self) {
        self.module.add_function(
            "Vec.destroy",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
    }

    /*

        DEFINITION FUNCTIONS (END)

    */
}
