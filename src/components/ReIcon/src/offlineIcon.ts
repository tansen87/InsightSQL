import { addIcon } from "@iconify/vue/dist/offline";

/**
 * 这里存放本地图标，在 src/layout/index.vue 文件中加载，避免在首启动加载
 */

// 本地菜单图标，后端在路由的icon中返回对应的图标字符串并且前端在此处使用addIcon添加即可渲染菜单图标
import SearchEyeLine from "@iconify-icons/ri/search-eye-line";
import Command from "@iconify-icons/ri/code-box-line";
import FlowChart from "@iconify-icons/ri/flow-chart";

addIcon("searchEyeLine", SearchEyeLine);
addIcon("command", Command);
addIcon("flowChart", FlowChart);
