#![allow(dead_code)]

use std::io::{Cursor, Read};
use std::sync::Mutex;
use std::{mem, slice};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;

use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::GrpAsset;
use crate::read;

lazy_static! {
    pub static ref DEBUG: Mutex<bool> = Mutex::new(false);
}

pub const ENTRY_SCENE: usize = 70;
pub const ENTRY_X: i16 = 19;
pub const ENTRY_Y: i16 = 20;

// CC.SceneXMin=11
pub const SCENE_X_MIN: i16 = 11;
pub const SCENE_Y_MIN: i16 = 11;
pub const SCENE_X_MAX: i16 = 47;
pub const SCENE_Y_MAX: i16 = 47;
// CC.NewPersonPic=3445;                      --开始主角pic
pub const NEW_PERSON: usize = 3445;
// CC.MWidth=480;       --主地图宽
const MAIN_WIDTH: usize = 480;
// CC.MHeight=480;      --主地图高
const MAIN_HEIGHT: usize = 480;
//CC.SWidth=64;     --子场景地图大小
pub const SCENE_WIDTH: usize = 64;
// CC.SHeight=64;
pub const SCENE_HEIGHT: usize = 64;
// CC.DNUM=200;       --D*每个场景的事件数
pub const DNUM: usize = 200;

pub const LAYER_NUM: usize = 6;

// CONFIG.XSCALE = 18    -- 贴图宽度的一半
pub const XSCALE: f32 = 18.;
// CONFIG.YSCALE = 9     -- 贴图高度的一半
pub const YSCALE: f32 = 9.;

// CC.XSCALE=CONFIG.XSCALE;    --贴图一半的宽高
// CC.YSCALE=CONFIG.YSCALE;

/*
CC.TEAM_NUM=6;          --队伍人数
CC.MY_THING_NUM=200      --主角物品数量
 */
const TEAM_NUM: usize = 6;
const MY_THING_NUM: usize = 200;
const ACTION_FRAME: usize = 5;
const SKILL_NUM: usize = 10;
const ITEM_NUM: usize = 4;

const PERSON_SIZE: usize = mem::size_of::<Person>();
const BASE_SIZE: usize = mem::size_of::<Base>();
const SCENE_SIZE: usize = mem::size_of::<Scene>();

#[derive(Debug)]
pub struct Base {
    // CC.Base_S["乘船"] ={0, 0, 2} - - 起始位置(从0开始)，数据类型(0有符号 1无符号，2字符串)，长度
    pub boat: i16,
    // CC.Base_S["无用"] ={2, 0, 2};
    pub useless: i16,
    // CC.Base_S["人X"] ={4, 0, 2};
    pub person_x: i16,
    // CC.Base_S["人Y"] ={6, 0, 2};
    pub person_y: i16,
    // CC.Base_S["人X1"] ={8, 0, 2};
    pub person_x_scene: i16,
    // CC.Base_S["人Y1"] ={10, 0, 2};
    pub person_y_scene: i16,
    // CC.Base_S["人方向"] ={12, 0, 2};
    pub person_dir: i16,
    // CC.Base_S["船X"] ={14, 0, 2};
    pub boat_x: i16,
    // CC.Base_S["船Y"] ={16, 0, 2};
    pub boat_y: i16,
    // CC.Base_S["船X1"] ={18, 0, 2};
    pub boat_x1: i16,
    // CC.Base_S["船Y1"] ={20, 0, 2};
    pub boat_y1: i16,
    // CC.Base_S["船方向"] ={22, 0, 2};
    pub boat_dir: i16,
    /*
    for i = 1,
    CC.TEAM_NUM do
    CC.Base_S["队伍"..i]={24 + 2 * (i - 1),0, 2};
    end
     */
    pub teams: [i16; TEAM_NUM],
    /*
    for i = 1,
    CC.MY_THING_NUM do
    CC.Base_S["物品"..i]={36 + 4 * (i - 1),0, 2};
    CC.Base_S["物品数量"..i]={36 + 4 * (i - 1) +2, 0, 2};
    end
     */
    pub items: [(i16, i16); MY_THING_NUM],
}

#[derive(Debug, Default)]
pub struct Backpack {
    pub items: Vec<(i32, i32)>,
}

#[derive(Debug)]
pub struct Scene {
    // CC.Scene_S["代号"]={0,0,2}
    code: i16,
    // CC.Scene_S["名称"]={2,2,20}
    name: [u8; 20],
    // CC.Scene_S["出门音乐"]={22,0,2}
    leave_music: i16,
    // CC.Scene_S["进门音乐"]={24,0,2}
    enter_music: i16,
    // CC.Scene_S["跳转场景"]={26,0,2}
    jump: i16,
    // CC.Scene_S["进入条件"]={28,0,2}
    enter_pre: i16,
    // CC.Scene_S["外景入口X1"]={30,0,2}
    out_entry_x1: i16,
    // CC.Scene_S["外景入口Y1"]={32,0,2}
    out_entry_y1: i16,
    // CC.Scene_S["外景入口X2"]={34,0,2}
    out_entry_x2: i16,
    // CC.Scene_S["外景入口Y2"]={36,0,2}
    out_entry_y2: i16,
    // CC.Scene_S["入口X"]={38,0,2}
    entry_x: i16,
    // CC.Scene_S["入口Y"]={40,0,2}
    entry_y: i16,
    // CC.Scene_S["出口X1"]={42,0,2}
    exit_x1: i16,
    // CC.Scene_S["出口X2"]={44,0,2}
    exit_x2: i16,
    // CC.Scene_S["出口X3"]={46,0,2}
    exit_x3: i16,
    // CC.Scene_S["出口Y1"]={48,0,2}
    exit_y1: i16,
    // CC.Scene_S["出口Y2"]={50,0,2}
    exit_y2: i16,
    // CC.Scene_S["出口Y3"]={52,0,2}
    exit_y3: i16,
    // CC.Scene_S["跳转口X1"]={54,0,2}
    jump_x1: i16,
    // CC.Scene_S["跳转口Y1"]={56,0,2}
    jump_y1: i16,
    // CC.Scene_S["跳转口X2"]={58,0,2}
    jump_x2: i16,
    // CC.Scene_S["跳转口Y2"]={60,0,2}
    jump_y2: i16,
}

#[repr(C)]
#[derive(Debug)]
pub struct Person {
    // CC.Person_S["代号"]={0,0,2}
    code: i16,
    // CC.Person_S["头像代号"]={2,0,2}
    avatar: i16,
    // CC.Person_S["生命增长"]={4,0,2}
    life_gain: i16,
    // CC.Person_S["无用"]={6,0,2}
    useless: i16,
    // CC.Person_S["姓名"]={8,2,20}
    name: [u8; 20],
    // CC.Person_S["外号"]={28,2,20}
    alias: [u8; 20],
    // CC.Person_S["性别"]={48,0,2}
    male: i16,
    // CC.Person_S["等级"]={50,0,2}
    level: i16,
    // CC.Person_S["经验"]={52,1,2}
    exp: u16,
    // CC.Person_S["生命"]={54,0,2}
    life: i16,
    // CC.Person_S["生命最大值"]={56,0,2}
    life_max: i16,
    // CC.Person_S["受伤程度"]={58,0,2}
    injure: i16,
    // CC.Person_S["中毒程度"]={60,0,2}
    tox: i16,
    // CC.Person_S["体力"]={62,0,2}
    vatity: i16,
    // CC.Person_S["物品修炼点数"]={64,0,2}
    item_familiar: i16,
    // CC.Person_S["武器"]={66,0,2}
    weapon: i16,
    // CC.Person_S["防具"]={68,0,2}
    armor: i16,
    // actions
    // for i=1,5 do
    // CC.Person_S["出招动画帧数" .. i]={70+2*(i-1),0,2};
    // CC.Person_S["出招动画延迟" .. i]={80+2*(i-1),0,2};
    // CC.Person_S["武功音效延迟" .. i]={90+2*(i-1),0,2};
    // end
    action_frames: [i16; ACTION_FRAME],
    action_delays: [i16; ACTION_FRAME],
    action_audio_delays: [i16; ACTION_FRAME],
    // CC.Person_S["内力性质"]={100,0,2}
    neili_status: i16,
    // CC.Person_S["内力"]={102,0,2}
    neili: i16,
    // CC.Person_S["内力最大值"]={104,0,2}
    neili_max: i16,
    // CC.Person_S["攻击力"] ={106, 0, 2}
    attack: i16,
    // CC.Person_S["轻功"] ={108, 0, 2}
    agile: i16,
    // CC.Person_S["防御力"] ={110, 0, 2}
    defence: i16,
    // CC.Person_S["医疗能力"] ={112, 0, 2}
    cure: i16,
    // CC.Person_S["用毒能力"] ={114, 0, 2}
    poison: i16,
    // CC.Person_S["解毒能力"] ={116, 0, 2}
    depoison: i16,
    // CC.Person_S["抗毒能力"] ={118, 0, 2}
    poison_def: i16,
    // CC.Person_S["拳掌功夫"] ={120, 0, 2}
    fist: i16,
    // CC.Person_S["御剑能力"] ={122, 0, 2}
    sword: i16,
    // CC.Person_S["耍刀技巧"] ={124, 0, 2}
    knife: i16,
    // CC.Person_S["特殊兵器"] ={126, 0, 2}
    other_weapon: i16,
    // CC.Person_S["暗器技巧"] ={128, 0, 2}
    fly_weapon: i16,
    // CC.Person_S["武学常识"] ={130, 0, 2}
    wknowlege: i16,
    // CC.Person_S["品德"] ={132, 0, 2}
    sanity: i16,
    // CC.Person_S["攻击带毒"] ={134, 0, 2}
    with_poison: i16,
    // CC.Person_S["左右互搏"] ={136, 0, 2}
    double_att: i16,
    // CC.Person_S["声望"] ={138, 0, 2}
    reputation: i16,
    // CC.Person_S["资质"] ={140, 0, 2}
    pub talent: i16,
    // CC.Person_S["修炼物品"] ={142, 0, 2}
    item_train: i16,
    // CC.Person_S["修炼点数"] ={144, 0, 2}
    item_point: i16,
    /*
    for i = 1,
    10 do
    CC.Person_S["武功"..i]={146 + 2 * (i - 1),0, 2};
    CC.Person_S["武功等级"..i]={166 + 2 * (i - 1),0, 2};
    end
     */
    skills: [i16; SKILL_NUM],
    skill_levels: [i16; SKILL_NUM],
    /*
    for i = 1,
    4 do
    CC.Person_S["携带物品"..i]={186 + 2 * (i - 1),0, 2};
    CC.Person_S["携带物品数量"..i]={194 + 2 * (i - 1),0, 2};
    end
     */
    items: [i16; ITEM_NUM],
    item_nums: [i16; ITEM_NUM],
}

fn to_str(v: &[u8]) -> String {
    let out = v
        .iter()
        .map_while(|v| if *v == 0 { None } else { Some(*v) })
        .collect::<Vec<u8>>();
    String::from_utf8(out).unwrap()
}

pub fn rbg2rgba(c: u32) -> u32 {
    ((c & 0xFF) << 16) + (c & 0xFF00) + ((c & 0xFF0000) >> 16) + 0xFF000000
}

impl Person {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        let code = read!(c, i16);
        let avatar = read!(c, i16);
        let life_gain = read!(c, i16);
        let useless = read!(c, i16);
        let mut name = [0; 20];
        c.read(&mut name).unwrap();
        let mut alias = [0; 20];
        c.read(&mut alias).unwrap();
        let male = read!(c, i16);
        let level = read!(c, i16);
        let exp = read!(c, u16);
        let life = read!(c, i16);
        let life_max = read!(c, i16);
        let injure = read!(c, i16);
        let tox = read!(c, i16);
        let vatity = read!(c, i16);
        let item_familiar = read!(c, i16);
        let weapon = read!(c, i16);
        let armor = read!(c, i16);
        let action_frames = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let action_delays = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let action_audio_delays = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let neili_status = read!(c, i16);
        let neili = read!(c, i16);
        let neili_max = read!(c, i16);
        let attack = read!(c, i16);
        let agile = read!(c, i16);
        let defence = read!(c, i16);
        let cure = read!(c, i16);
        let poison = read!(c, i16);
        let depoison = read!(c, i16);
        let poison_def = read!(c, i16);
        let fist = read!(c, i16);
        let sword = read!(c, i16);
        let knife = read!(c, i16);
        let other_weapon = read!(c, i16);
        let fly_weapon = read!(c, i16);
        let wknowlege = read!(c, i16);
        let sanity = read!(c, i16);
        let with_poison = read!(c, i16);
        let double_att = read!(c, i16);
        let reputation = read!(c, i16);
        let talent = read!(c, i16);
        let item_train = read!(c, i16);
        let item_point = read!(c, i16);
        let skills = [0; SKILL_NUM].map(|_| read!(c, i16));
        let skill_levels = [0; SKILL_NUM].map(|_| read!(c, i16));
        let items = [0; ITEM_NUM].map(|_| read!(c, i16));
        let item_nums = [0; ITEM_NUM].map(|_| read!(c, i16));
        Person {
            code,
            avatar,
            life_gain,
            useless,
            name,
            alias,
            male,
            level,
            exp,
            life,
            life_max,
            injure,
            tox,
            vatity,
            item_familiar,
            weapon,
            armor,
            action_frames,
            action_delays,
            action_audio_delays,
            neili_status,
            neili,
            neili_max,
            attack,
            agile,
            defence,
            cure,
            poison,
            depoison,
            poison_def,
            fist,
            sword,
            knife,
            other_weapon,
            fly_weapon,
            wknowlege,
            sanity,
            with_poison,
            double_att,
            reputation,
            talent,
            item_train,
            item_point,
            skills,
            skill_levels,
            items,
            item_nums,
        }
    }

    pub fn name(&self) -> String {
        to_str(&self.name)
    }

    pub fn alias(&self) -> String {
        to_str(&self.alias)
    }
}

impl Base {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        Base {
            boat: read!(c, i16),
            useless: read!(c, i16),
            person_x: read!(c, i16),
            person_y: read!(c, i16),
            person_x_scene: read!(c, i16),
            person_y_scene: read!(c, i16),
            person_dir: read!(c, i16),
            boat_x: read!(c, i16),
            boat_y: read!(c, i16),
            boat_x1: read!(c, i16),
            boat_y1: read!(c, i16),
            boat_dir: read!(c, i16),
            teams: [0; TEAM_NUM].map(|_| read!(c, i16)),
            items: [0; MY_THING_NUM].map(|_| (read!(c, i16), read!(c, i16))),
        }
    }
}

impl Scene {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        Scene {
            code: read!(c, i16),
            name: {
                let mut name = [0; 20];
                c.read(&mut name).unwrap();
                name
            },
            leave_music: read!(c, i16),
            enter_music: read!(c, i16),
            jump: read!(c, i16),
            enter_pre: read!(c, i16),
            out_entry_x1: read!(c, i16),
            out_entry_y1: read!(c, i16),
            out_entry_x2: read!(c, i16),
            out_entry_y2: read!(c, i16),
            entry_x: read!(c, i16),
            entry_y: read!(c, i16),
            exit_x1: read!(c, i16),
            exit_x2: read!(c, i16),
            exit_x3: read!(c, i16),
            exit_y1: read!(c, i16),
            exit_y2: read!(c, i16),
            exit_y3: read!(c, i16),
            jump_x1: read!(c, i16),
            jump_y1: read!(c, i16),
            jump_x2: read!(c, i16),
            jump_y2: read!(c, i16),
        }
    }

    pub fn name(&self) -> String {
        to_str(&self.name)
    }
}

pub struct SData(Vec<u8>);

impl SData {
    pub fn new(gs: GrpAsset) -> Self {
        SData(gs.data)
    }

    pub fn get_texture(&self, scene_id: usize, w: usize, h: usize, layer: usize) -> i16 {
        let i = (scene_id * LAYER_NUM + layer) * SCENE_WIDTH * SCENE_HEIGHT + h * SCENE_WIDTH + w;
        let mut data = &self.0[i * 2..];
        data.read_i16::<LittleEndian>().unwrap()
    }
}

pub struct DData(Vec<u8>);

impl DData {
    pub fn new(gs: GrpAsset) -> Self {
        DData(gs.data)
    }

    pub fn get_d(&self, scene_id: usize, id: usize, i: usize) -> i16 {
        let i = (scene_id * DNUM + id) * 11 + i;
        let mut data = &self.0[i * 2..];
        data.read_i16::<LittleEndian>().unwrap()
    }

    pub fn set(&mut self, scene_id: usize, id: usize, i: usize, val: i16) {
        let i = (scene_id * DNUM + id) * 11 + i;
        let data = &mut self.0[i * 2..i * 2 + 2];
        let mut test = vec![];
        test.write_i16::<LittleEndian>(val).unwrap();
        data[0] = test[0];
        data[1] = test[1];
    }
}

pub struct TextureMap {
    pub gs: GrpAsset,
}

fn parse(
    buf: &mut Vec<u32>,
    w: usize,
    mut c: impl std::io::Read,
    colors: &Vec<u32>,
) -> std::io::Result<()> {
    let mut _idx = 0;
    let mut row_num = 0;
    while let Ok(mut row_left) = c.read_u8() {
        /*
        if *DEBUG.lock().unwrap() {
            println!("row:{} {} ", row_num, row_left);
        }
         */
        _idx = row_num * w;
        row_num += 1;

        while row_left > 0 {
            let empty = c.read_u8()?;
            /*
            if *DEBUG.lock().unwrap() {
                println!("skip {}", empty);
            }
             */
            _idx += empty as usize;
            row_left -= 2;

            if row_left > 0 {
                let next = c.read_u8()?;
                /*
                if *DEBUG.lock().unwrap() {
                    println!("read {}", next);
                }
                 */
                row_left -= next;
                for _ in 0..next {
                    let c = c.read_u8()?;
                    /*
                    if *DEBUG.lock().unwrap() {
                        println!("read solid {}", c);
                    }
                     */
                    if let Some(v) = buf.get_mut(_idx as usize) {
                        let c = *colors.get(c as usize).unwrap();
                        *v = c;
                        _idx += 1;
                    }
                }
            }
        }
    }
    Ok(())
}

// width, heigh, xoff, yoff
#[derive(Copy, Clone, Debug)]
pub struct TextureMeta(
    pub u16, // width
    pub u16, // height
    pub f32, // xoff
    pub f32, // yoff
);

impl TextureMap {
    pub fn new(gs: GrpAsset) -> TextureMap {
        TextureMap { gs }
    }

    pub fn get_image(&self, id: usize, colors: &Vec<u32>) -> Option<(Image, TextureMeta)> {
        let maybe_data = self.gs.idx(id as usize);
        if maybe_data.is_none() {
            return None;
        }
        let data = maybe_data.unwrap();
        let mut c = Cursor::new(data);
        let w = read!(c, u16);
        let h = read!(c, u16);
        let xoff = read!(c, i16);
        let yoff = read!(c, i16);
        if *DEBUG.lock().unwrap() {
            println!(
                "data len {} w:{}, h:{}, xoff:{}, yoff:{}",
                data.len(),
                w,
                h,
                xoff,
                yoff
            );
        }

        let mut decode_buf: Vec<u32> = vec![];
        decode_buf.resize((w * h) as usize, 0x0);

        parse(&mut decode_buf, w as usize, &mut c, colors).unwrap();
        let pixel_ptr = decode_buf.as_ptr() as *const u8;
        let pixel = unsafe { slice::from_raw_parts(pixel_ptr, decode_buf.len() * 4) };

        let image = Image::new_fill(
            Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixel,
            TextureFormat::Rgba8UnormSrgb,
        );

        Some((image, TextureMeta(w, h, xoff as f32, yoff as f32)))
    }
}

#[derive(Clone)]
pub struct Palette(pub Vec<u32>);

pub fn load_color(data: &[u8]) -> Palette {
    Palette(
        data.chunks(3)
            .map(|v| {
                let mut c = Cursor::new(v);
                let c1 = read!(c, u8) as u32;
                let c2 = read!(c, u8) as u32;
                let c3 = read!(c, u8) as u32;
                let rgb = (c1 << 18) + (c2 << 10) + (c3 << 2);
                rbg2rgba(rgb)
            })
            .collect(),
    )
}

#[repr(C)]
#[derive(Debug)]
pub struct Thing {
    // CC.Thing_S["代号"]={0,0,2}
    code: i16,
    // CC.Thing_S["名称"]={2,2,40}
    name: [u8; 40],
    // CC.Thing_S["名称2"]={42,2,40}
    name2: [u8; 40],
    // CC.i16_S["物品说明"]={82,2,60}
    desp: [u8; 60],
    // CC.Thing_S["练出武功"]={142,0,2}
    wugong: i16,
    // CC.Thing_S["暗器动画编号"]={144,0,2}
    animate_code: i16,
    // CC.Thing_S["使用人"]={146,0,2}
    user: i16,
    // CC.Thing_S["装备类型"]={148,0,2}
    equip_type: i16,
    // CC.Thing_S["显示物品说明"]={150,0,2}
    show: i16,
    // CC.Thing_S["类型"]={152,0,2}
    typ: i16,
    // CC.Thing_S["未知5"]={154,0,2}
    unknown5: i16,
    // CC.Thing_S["未知6"]={156,0,2}
    unknown6: i16,
    // CC.Thing_S["未知7"]={158,0,2}
    unknown7: i16,
    // CC.Thing_S["加生命"]={160,0,2}
    add_life: i16,
    // CC.Thing_S["加生命最大值"]={162,0,2}
    add_max_life: i16,
    // CC.Thing_S["加中毒解毒"]={164,0,2}
    add_detox: i16,
    // CC.Thing_S["加体力"]={166,0,2}
    add_vat: i16,
    // CC.Thing_S["改变内力性质"]={168,0,2}
    change_mag: i16,
    // CC.Thing_S["加内力"]={170,0,2}
    add_mag: i16,
    // CC.Thing_S["加内力最大值"]={172,0,2}
    add_max_mag: i16,
    // CC.Thing_S["加攻击力"]={174,0,2}
    add_attack: i16,
    // CC.Thing_S["加轻功"]={176,0,2}
    add_agile: i16,
    // CC.Thing_S["加防御力"]={178,0,2}
    add_def: i16,
    // CC.Thing_S["加医疗能力"]={180,0,2}
    add_heal: i16,
    // CC.Thing_S["加用毒能力"]={182,0,2}
    add_poison: i16,
    // CC.Thing_S["加解毒能力"]={184,0,2}
    add_depoison: i16,
    // CC.Thing_S["加抗毒能力"]={186,0,2}
    add_anti_poi: i16,
    // CC.Thing_S["加拳掌功夫"]={188,0,2}
    add_fist: i16,
    // CC.Thing_S["加御剑能力"]={190,0,2}
    add_sword: i16,
    // CC.Thing_S["加耍刀技巧"]={192,0,2}
    add_knife: i16,
    // CC.Thing_S["加特殊兵器"]={194,0,2}
    add_special_weapon: i16,
    // CC.Thing_S["加暗器技巧"]={196,0,2}
    add_arrow: i16,
    // CC.Thing_S["加武学常识"]={198,0,2}
    add_know: i16,
    // CC.Thing_S["加品德"]={200,0,2}
    add_rep: i16,
    // CC.Thing_S["加攻击次数"]={202,0,2}
    add_attack_time: i16,
    // CC.Thing_S["加攻击带毒"]={204,0,2}
    add_attack_poi: i16,
    // CC.Thing_S["仅修炼人物"]={206,0,2}
    use_only: i16,
    // CC.Thing_S["需内力性质"]={208,0,2}
    magic_type: i16,
    // CC.Thing_S["需内力"]={210,0,2}
    magic_req: i16,
    // CC.Thing_S["需攻击力"]={212,0,2}
    att_req: i16,
    // CC.Thing_S["需轻功"]={214,0,2}
    agile_req: i16,
    // CC.Thing_S["需用毒能力"]={216,0,2}
    poi_req: i16,
    // CC.Thing_S["需医疗能力"]={218,0,2}
    heal_req: i16,
    // CC.Thing_S["需解毒能力"]={220,0,2}
    depoi_req: i16,
    // CC.Thing_S["需拳掌功夫"]={222,0,2}
    fist_req: i16,
    // CC.i16_S["需御剑能力"]={224,0,2}
    sword_req: i16,
    // CC.Thing_S["需耍刀技巧"]={226,0,2}
    knife_req: i16,
    // CC.Thing_S["需特殊兵器"]={228,0,2}
    sp_weapon_req: i16,
    // CC.Thing_S["需暗器技巧"]={230,0,2}
    arrow_req: i16,
    // CC.Thing_S["需资质"]={232,0,2}
    ability_req: i16,
    // CC.Thing_S["需经验"]={234,0,2}
    exp_req: i16,
    // CC.Thing_S["练出物品需经验"]={236,0,2}
    item_exp: i16,
    // CC.Thing_S["需材料"]={238,0,2}
    meterial_req: i16,
    items: [i16; 5],
    item_reqs: [i16; 5],
}

impl Thing {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        let code = read!(c, i16);
        let mut name = [0; 40];
        c.read(&mut name).unwrap();
        let mut name2 = [0; 40];
        c.read(&mut name2).unwrap();
        let mut desp = [0; 60];
        c.read(&mut desp).unwrap();
        let wugong = read!(c, i16);
        let animate_code = read!(c, i16);
        let user = read!(c, i16);
        let equip_type = read!(c, i16);
        let show = read!(c, i16);
        let typ = read!(c, i16);
        let unknown5 = read!(c, i16);
        let unknown6 = read!(c, i16);
        let unknown7 = read!(c, i16);
        let add_life = read!(c, i16);
        let add_max_life = read!(c, i16);
        let add_detox = read!(c, i16);
        let add_vat = read!(c, i16);
        let change_mag = read!(c, i16);
        let add_mag = read!(c, i16);
        let add_max_mag = read!(c, i16);
        let add_attack = read!(c, i16);
        let add_agile = read!(c, i16);
        let add_def = read!(c, i16);
        let add_heal = read!(c, i16);
        let add_poison = read!(c, i16);
        let add_depoison = read!(c, i16);
        let add_anti_poi = read!(c, i16);
        let add_fist = read!(c, i16);
        let add_sword = read!(c, i16);
        let add_knife = read!(c, i16);
        let add_special_weapon = read!(c, i16);
        let add_arrow = read!(c, i16);
        let add_know = read!(c, i16);
        let add_rep = read!(c, i16);
        let add_attack_time = read!(c, i16);
        let add_attack_poi = read!(c, i16);
        let use_only = read!(c, i16);
        let magic_type = read!(c, i16);
        let magic_req = read!(c, i16);
        let att_req = read!(c, i16);
        let agile_req = read!(c, i16);
        let poi_req = read!(c, i16);
        let heal_req = read!(c, i16);
        let depoi_req = read!(c, i16);
        let fist_req = read!(c, i16);
        let sword_req = read!(c, i16);
        let knife_req = read!(c, i16);
        let sp_weapon_req = read!(c, i16);
        let arrow_req = read!(c, i16);
        let ability_req = read!(c, i16);
        let exp_req = read!(c, i16);
        let item_exp = read!(c, i16);
        let meterial_req = read!(c, i16);
        let mut items = [0i16; 5];
        let mut item_reqs = [0i16; 5];
        for i in 0..items.len() {
            items[i] = read!(c, i16);
            item_reqs[i] = read!(c, i16);
        }
        Thing {
            code,
            name,
            name2,
            desp,
            wugong,
            animate_code,
            user,
            equip_type,
            show,
            typ,
            unknown5,
            unknown6,
            unknown7,
            add_life,
            add_max_life,
            add_detox,
            add_vat,
            change_mag,
            add_mag,
            add_max_mag,
            add_attack,
            add_agile,
            add_def,
            add_heal,
            add_poison,
            add_depoison,
            add_anti_poi,
            add_fist,
            add_sword,
            add_knife,
            add_special_weapon,
            add_arrow,
            add_know,
            add_rep,
            add_attack_time,
            add_attack_poi,
            use_only,
            magic_type,
            magic_req,
            att_req,
            agile_req,
            poi_req,
            heal_req,
            depoi_req,
            fist_req,
            sword_req,
            knife_req,
            sp_weapon_req,
            arrow_req,
            ability_req,
            exp_req,
            item_exp,
            meterial_req,
            items,
            item_reqs,
        }
    }

    pub fn name(&self) -> String {
        to_str(&self.name)
    }

    pub fn name2(&self) -> String {
        to_str(&self.name2)
    }

    pub fn desp(&self) -> String {
        to_str(&self.desp)
    }
}

pub struct Wugong([u8; 146]);

pub struct Shop([u8; 30]);
