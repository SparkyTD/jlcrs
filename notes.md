# Shape Geometry Types
___

### Polygon
```
[<start_x>,<start_y>, "L", <pt1_x>,<pt1_y>, <pt2_x>,<pt2_y>, ...]
```

## Rectangle
```
["R", <start_x>,<start_y>, <width>,<height>, <rotation>, <corner_radius>]
```

### Circle
```
["CIRCLE", <center_x>,<center_y>, <radius>]
```

### Arc
```
[<start_x>,<start_y>, "ARC", <end_x>,<end_y>, <rotation>]
```

### Center Arc
```
[<start_x>,<start_y>, "CARC", <rotation>, <end_x>,<end_y>]
```

# STEP Model Acquisition
```
lcsc_id = C93168
device_data = https://pro.easyeda.com/api/eda/product/search?keyword=$lcsc_id&currPage=1&pageSize=1
model_id = device_data[$.result.productList[0].device_info.footprint_info.model_3d.uri]
model_data = https://pro.easyeda.com/api/v2/components/$model_id
model_uuid = model_data[$.result.3d_model_uuid]
step_url = https://modules.easyeda.com/qAxj6KHrDKw4blvCG8QJPs7Y/$model_uuid
```