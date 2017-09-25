use std::sync::Arc;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::marker::PhantomData;
use libc::c_char;
use smallvec::SmallVec;
use vks;
use ::{VooResult, Instance, Surface, PhysicalDevice, SwapchainSupportDetails,
    DeviceQueueCreateInfo, CharStrs, PhysicalDeviceFeatures};
use queue::{self, Queue};
use instance;


#[derive(Debug)]
struct Inner {
    handle: vks::VkDevice,
    physical_device: PhysicalDevice,
    // features: vks::VkPhysicalDeviceFeatures,
    // queues: SmallVec<[u32; 32]>,
    queue_family_indexes: SmallVec<[u32; 16]>,
    // vk: vks::VkDevicePointers,
    instance: Instance,
    loader: vks::DeviceProcAddrLoader,
}

#[derive(Debug, Clone)]
pub struct Device {
    inner: Arc<Inner>,
}

impl Device {
    /// Returns a new `DeviceBuilder`.
    pub fn builder<'db>() -> DeviceBuilder<'db> {
        DeviceBuilder::new()
    }


    // // pub fn new(instance: Instance, surface: &Surface, physical_device: vks::VkPhysicalDevice,
    // //         queue_familiy_flags: vks::VkQueueFlags) -> VooResult<Device>
    // pub fn new(instance: Instance, physical_device: PhysicalDevice,
    //         create_info: vks::VkDeviceCreateInfo, queue_family_idx: u32) -> VooResult<Device> {

    //     // let create_info = vks::VkDeviceCreateInfo {
    //     //     sType: vks::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
    //     //     pNext: ptr::null(),
    //     //     flags: 0,
    //     //     queueCreateInfoCount: 1,
    //     //     pQueueCreateInfos: &queue_create_info,
    //     //     enabledLayerCount: enabled_layer_name_ptrs.len() as u32,
    //     //     ppEnabledLayerNames: enabled_layer_name_ptrs.as_ptr(),
    //     //     enabledExtensionCount: enabled_extension_name_ptrs.len() as u32,
    //     //     ppEnabledExtensionNames: enabled_extension_name_ptrs.as_ptr(),
    //     //     pEnabledFeatures: &features,
    //     // };

    //     // Device:
    //     let mut handle = ptr::null_mut();
    //     unsafe {
    //         ::check(instance.proc_addr_loader().core.vkCreateDevice(physical_device.handle(),
    //             &create_info, ptr::null(), &mut handle));
    //     }

    //     let mut loader = vks::DeviceProcAddrLoader::from_get_device_proc_addr(
    //         instance.proc_addr_loader().core.pfn_vkGetDeviceProcAddr);

    //     unsafe {
    //         loader.load_core(handle);
    //         // create_info.enabled_extensions.load_device(&mut loader, handle);
    //         // instance.loader().get_enabled_extensions().load_device(&mut loader, handle);
    //         // loader.load_khr_sampler_mirror_clamp_to_edge(handle);
    //         // loader.load_khr_draw_parameters(handle);
    //         loader.load_khr_swapchain(handle);
    //         // loader.load_khr_maintenance1(handle);
    //         // loader.load_amd_rasterization_order(handle);
    //         // loader.load_amd_draw_indirect_count(handle);
    //         // loader.load_amd_shader_ballot(handle);
    //         // loader.load_amd_shader_trinary_minmax(handle);
    //         // loader.load_amd_shader_explicit_vertex_parameter(handle);
    //         // loader.load_amd_gcn_shader(handle);
    //         // loader.load_amd_draw_indirect_count(handle);
    //         // loader.load_amd_negative_viewport_height(handle);
    //         // loader.load_amd_shader_info(handle);
    //         // loader.load_amd_wave_limits(handle);
    //         // loader.load_amd_texture_gather_bias_lod(handle);
    //         // loader.load_amd_programmable_sample_locations(handle);
    //         // loader.load_amd_mixed_attachment_samples(handle);
    //         // loader.load_ext_shader_subgroup_vote(handle);
    //         // loader.load_amd_gpa_interface(handle);
    //         // loader.load_ext_shader_subgroup_ballot(handle);
    //     }


    //     Ok(Device {
    //         inner: Arc::new(Inner {
    //             handle,
    //             physical_device,
    //             // features,
    //             queue_family_indexes,
    //             instance,
    //             loader,
    //         }),
    //     })
    // }

    #[inline]
    pub fn queue(&self, queue_idx: u32) -> vks::VkQueue {
        let mut queue_handle = ptr::null_mut();
        assert!(self.inner.queue_family_indexes.len() == 1,
            "Update this shitty queue family code.");
        unsafe {
            self.proc_addr_loader().core.vkGetDeviceQueue(self.inner.handle,
                self.inner.queue_family_indexes[0], queue_idx,
                &mut queue_handle);
        }
        queue_handle
    }

    #[inline]
    pub fn proc_addr_loader(&self) -> &vks::DeviceProcAddrLoader {
        // &self.inner.vk
        &self.inner.loader
    }

    #[inline]
    pub fn handle(&self) -> vks::VkDevice {
        self.inner.handle
    }

    #[inline]
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.inner.physical_device
    }

    #[inline]
    pub fn instance(&self) -> &Instance {
        &self.inner.instance
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Destroying device...");
        unsafe {
            self.instance.proc_addr_loader().core.vkDestroyDevice(self.handle, ptr::null());
        }
    }
}


// typedef struct VkDeviceCreateInfo {
//     VkStructureType                    sType;
//     const void*                        pNext;
//     VkDeviceCreateFlags                flags;
//     uint32_t                           queueCreateInfoCount;
//     const VkDeviceQueueCreateInfo*     pQueueCreateInfos;
//     uint32_t                           enabledLayerCount;
//     const char* const*                 ppEnabledLayerNames;
//     uint32_t                           enabledExtensionCount;
//     const char* const*                 ppEnabledExtensionNames;
//     const VkPhysicalDeviceFeatures*    pEnabledFeatures;
// } VkDeviceCreateInfo;
//
#[derive(Debug, Clone)]
pub struct DeviceBuilder<'db> {
    create_info: vks::VkDeviceCreateInfo,
    enabled_layer_names: Option<CharStrs<'db>>,
    enabled_extension_names: Option<CharStrs<'db>>,
    _p: PhantomData<&'db ()>,
}

impl<'db> DeviceBuilder<'db> {
    /// Returns a new instance builder.
    pub fn new() -> DeviceBuilder<'db> {
        DeviceBuilder {
            create_info: vks::VkDeviceCreateInfo::default(),
            enabled_layer_names: None,
            enabled_extension_names: None,
            _p: PhantomData,
        }
    }

    /// Specifies the queue_creation info.
    pub fn queue_create_infos<'s, 'ci>(&'s mut self,
            queue_create_infos: &'ci [DeviceQueueCreateInfo])
            -> &'s mut DeviceBuilder<'db>
            where 'ci: 'db {
        self.create_info.queueCreateInfoCount = queue_create_infos.len() as u32;
        debug_assert_eq!(mem::align_of::<DeviceQueueCreateInfo>(),
            mem::align_of::<vks::VkDeviceQueueCreateInfo>());
        debug_assert_eq!(mem::size_of::<DeviceQueueCreateInfo>(),
            mem::size_of::<vks::VkDeviceQueueCreateInfo>());
        self.create_info.pQueueCreateInfos = queue_create_infos.as_ptr() as *const _;
        self
    }

    /// Specifies the layer names to enable.
    pub fn enabled_layer_names<'s, 'cs, Cs>(&'s mut self, enabled_layer_names: Cs)
            -> &'s mut DeviceBuilder<'db>
            where 'cs: 'db, Cs: 'cs + Into<CharStrs<'cs>> {
        let enabled_layer_names = enabled_layer_names.into();
        self.create_info.enabledLayerCount = enabled_layer_names.len() as u32;
        self.create_info.ppEnabledLayerNames = enabled_layer_names.as_ptr() as *const _;
        self.enabled_layer_names = Some(enabled_layer_names);
        self
    }

    /// Specifies the extension names to enable.
    pub fn enabled_extension_names<'s, 'cs, Cs>(&'s mut self, enabled_extension_names: Cs)
            -> &'s mut DeviceBuilder<'db>
            where 'cs: 'db, Cs: 'cs + Into<CharStrs<'cs>> {
        let enabled_extension_names = enabled_extension_names.into();
        self.create_info.enabledExtensionCount = enabled_extension_names.len() as u32;
        self.create_info.ppEnabledExtensionNames = enabled_extension_names.as_ptr() as *const _;
        self.enabled_extension_names = Some(enabled_extension_names);
        self
    }

    pub fn enabled_features<'s, 'f>(&'s mut self, enabled_features: &'f PhysicalDeviceFeatures)
            -> &'s mut DeviceBuilder<'db>
            where 'f: 'db {
        self.create_info.pEnabledFeatures = enabled_features.raw();
        self
    }

    pub fn build(&self, physical_device: PhysicalDevice) -> VooResult<Device> {
                // Device:
        let mut handle = ptr::null_mut();
        unsafe {
            ::check(physical_device.instance().proc_addr_loader().core.vkCreateDevice(physical_device.handle(),
                &self.create_info, ptr::null(), &mut handle));
        }

        let mut loader = vks::DeviceProcAddrLoader::from_get_device_proc_addr(
            physical_device.instance().proc_addr_loader().core.pfn_vkGetDeviceProcAddr);

        unsafe {
            loader.load_core(handle);
            // create_info.enabled_extensions.load_device(&mut loader, handle);
            // instance.loader().get_enabled_extensions().load_device(&mut loader, handle);
            // loader.load_khr_sampler_mirror_clamp_to_edge(handle);
            // loader.load_khr_draw_parameters(handle);
            loader.load_khr_swapchain(handle);
            // loader.load_khr_maintenance1(handle);
            // loader.load_amd_rasterization_order(handle);
            // loader.load_amd_draw_indirect_count(handle);
            // loader.load_amd_shader_ballot(handle);
            // loader.load_amd_shader_trinary_minmax(handle);
            // loader.load_amd_shader_explicit_vertex_parameter(handle);
            // loader.load_amd_gcn_shader(handle);
            // loader.load_amd_draw_indirect_count(handle);
            // loader.load_amd_negative_viewport_height(handle);
            // loader.load_amd_shader_info(handle);
            // loader.load_amd_wave_limits(handle);
            // loader.load_amd_texture_gather_bias_lod(handle);
            // loader.load_amd_programmable_sample_locations(handle);
            // loader.load_amd_mixed_attachment_samples(handle);
            // loader.load_ext_shader_subgroup_vote(handle);
            // loader.load_amd_gpa_interface(handle);
            // loader.load_ext_shader_subgroup_ballot(handle);
        }

        let instance = physical_device.instance().clone();
        let mut queue_family_indexes = SmallVec::<[u32; 16]>::new();
        for i in 0..(self.create_info.queueCreateInfoCount as isize) {
            unsafe {
                let queue_create_info_ptr = self.create_info.pQueueCreateInfos.offset(i);
                queue_family_indexes.push((*queue_create_info_ptr).queueFamilyIndex);
            }
        }
        assert!(queue_family_indexes.len() == 1, "Update this shitty queue family code.");

        Ok(Device {
            inner: Arc::new(Inner {
                handle,
                physical_device,
                // features,
                queue_family_indexes: queue_family_indexes,
                instance,
                loader,
            }),
        })
    }
}