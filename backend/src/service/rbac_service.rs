// Facade: re-export 共享类型，各子模块处理实际业务
#[allow(unused_imports)]
pub use crate::dto::{MenuNode, RbacInfo, PageQuery, PagedResult, I18nKv, DictWithItems};
#[allow(unused_imports)]
pub use crate::service::sys_dept_service::{DeptNode, list_dept_tree};
#[allow(unused_imports)]
pub use crate::service::sys_user_service::{UserListItem, list_users_paged, create_user, update_user,
    delete_user, reset_password, get_user, get_user_role_ids, get_rbac_info, user_is_super_admin, is_admin_user};
#[allow(unused_imports)]
pub use crate::service::sys_role_service::{list_roles_paged, list_roles, create_role, update_role,
    delete_role, set_role_menus, get_role_menus};
#[allow(unused_imports)]
pub use crate::service::sys_dept_service::{list_depts_paged, list_depts, create_dept, update_dept, delete_dept};
#[allow(unused_imports)]
pub use crate::service::sys_post_service::{list_posts_paged, list_posts, create_post, update_post, delete_post};
#[allow(unused_imports)]
pub use crate::service::sys_menu_service::{list_menus, create_menu, update_menu, delete_menus, delete_menu};
#[allow(unused_imports)]
pub use crate::service::sys_dict_service::{list_dicts, get_dict, create_dict, update_dict, delete_dict,
    list_dict_items, get_dict_with_items, create_dict_item, update_dict_item, delete_dict_item};
#[allow(unused_imports)]
pub use crate::service::sys_i18n_service::{list_i18n_locales, list_i18n, list_i18n_paged,
    upsert_i18n_batch, delete_i18n, get_i18n_messages};
