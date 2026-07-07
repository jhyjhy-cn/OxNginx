; 卸载时清理运行时数据目录
!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$INSTDIR\backup"
  RMDir /r "$INSTDIR\configs"
  RMDir /r "$INSTDIR\datas"
  RMDir /r "$INSTDIR\server"
  RMDir /r "$INSTDIR\ssl"
  RMDir /r "$INSTDIR\wwwlogs"
  RMDir /r "$INSTDIR\wwwroot"
  RMDir /r "$INSTDIR\logs"
  RMDir "$INSTDIR"
!macroend
