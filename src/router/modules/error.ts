export default {
  path: "/error",
  redirect: "/error/md",
  meta: {
    icon: "informationLine",
    title: "tips",
    // showLink: false,
    rank: 9
  },
  children: [
    {
      path: "/error/md",
      name: "md",
      component: () => import("@/views/error/md.vue"),
      meta: {
        title: "tips"
      }
    }
  ]
} as RouteConfigsTable;
