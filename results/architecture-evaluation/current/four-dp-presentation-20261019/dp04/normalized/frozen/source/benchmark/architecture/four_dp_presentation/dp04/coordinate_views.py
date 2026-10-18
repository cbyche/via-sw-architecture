"""A5 mechanical raw-event views. Never reads SVG, labels inventory or oracle."""
import copy, hashlib, json, math, time
from pathlib import Path

PADDING = 128
GRID_STEP = 100

def specs(revision):
    out=[]
    for frame in revision['frames']:
        w,h=frame['width'],frame['height'];points=[]
        for event in revision['events']:
            if event.get('frame_id')!=frame['frame_id']:continue
            if event['type']=='pointer_move':points.append((event['x'],event['y']))
            elif event['type']=='pointer_path':points.extend(event['points'])
        # Navigation drag is not a designated region. With no designation cue,
        # retain the entire frame instead of inferring a target from a bad bbox.
        box=(0,0,w,h)
        if points:
            box=(max(0,math.floor(min(p[0] for p in points))-PADDING),max(0,math.floor(min(p[1] for p in points))-PADDING),min(w,math.ceil(max(p[0] for p in points))+PADDING),min(h,math.ceil(max(p[1] for p in points))+PADDING))
        out.append({'view_id':'coordinate-'+frame['frame_id'],'frame_id':frame['frame_id'],'source_path':frame['image_path'],'source_sha256':frame['sha256'],'crop_original_pixels':list(box),'view_width':box[2]-box[0],'view_height':box[3]-box[1],'view_to_original':{'scale_x':1,'scale_y':1,'offset_x':box[0],'offset_y':box[1]},'grid_step':GRID_STEP,'rule':'raw pointer/path union +128px; no cue => full frame; no resize'})
    return out

def render(revision,folder):
    from PIL import Image,ImageDraw
    started=time.monotonic_ns();folder=Path(folder);folder.mkdir(parents=True,exist_ok=True)
    views=specs(revision)
    for v in views:
        source=Path(v['source_path'])
        if hashlib.sha256(source.read_bytes()).hexdigest()!=v['source_sha256']:raise ValueError('Source image SHA mismatch')
        image=Image.open(source).convert('RGB').crop(v['crop_original_pixels']);draw=ImageDraw.Draw(image,'RGBA')
        x0,y0,x1,y1=v['crop_original_pixels']
        for x in range(math.ceil(x0/GRID_STEP)*GRID_STEP,x1,GRID_STEP):
            draw.line([(x-x0,0),(x-x0,y1-y0-1)],fill=(0,90,220,65),width=1)
            draw.text((x-x0+2,2),str(x),fill=(0,35,145,255),stroke_width=1,stroke_fill=(255,255,255,230))
        for y in range(math.ceil(y0/GRID_STEP)*GRID_STEP,y1,GRID_STEP):
            draw.line([(0,y-y0),(x1-x0-1,y-y0)],fill=(0,90,220,65),width=1)
            draw.text((2,y-y0+2),str(y),fill=(0,35,145,255),stroke_width=1,stroke_fill=(255,255,255,230))
        path=folder/(v['view_id']+'.png');image.save(path)
        v['path']=str(path.resolve());v['sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
    return views,{'start_ns':started,'end_ns':time.monotonic_ns(),'scope':'actual post-input recovery rendering; counted in callback time/RSS; no model/candidate answers used'}

def map_extents(result,views):
    """Only explicit view-coordinate extents transform; originals pass unchanged."""
    out=copy.deepcopy(result);mapping=[];known={v['view_id']:v for v in views}
    for adoption in out.get('adoptions',[]):
        for extent in adoption.get('extents',[]):
            if extent.get('coordinate_space')!='view_pixels':continue
            view=known.get(extent.get('view_id'))
            if view is None or extent.get('frame_id')!=view['frame_id']:raise ValueError('Unknown/mismatched coordinate view')
            box=extent.get('bbox')
            if not isinstance(box,list) or len(box)!=4 or any(type(n) not in (int,float) or not math.isfinite(n) for n in box):raise ValueError('Invalid view bbox')
            if not 0<=box[0]<box[2]<=view['view_width'] or not 0<=box[1]<box[3]<=view['view_height']:raise ValueError('View bbox outside actual crop; no clamping')
            tx=view['view_to_original'];original=[box[0]+tx['offset_x'],box[1]+tx['offset_y'],box[2]+tx['offset_x'],box[3]+tx['offset_y']]
            mapping.append({'view_id':view['view_id'],'view_bbox':box,'original_bbox':original,'transform':tx})
            extent['bbox']=original;extent['coordinate_space']='frame_pixels'
    return out,mapping
