"""Fixed model-free controls for the deferred moderate-prefix proposal."""

PARAGRAPH='夕暮れの港にある資料室で古い地図を広げると、細い川と石造りの橋が描かれていました。窓の外では船の灯がゆっくり動き、通りの店から温かな湯気が流れてきます。友人と記録帳を読み比べ、失われた駅の場所や道の名前を確かめました。帰りには坂の途中で足を止め、遠くの鐘の音を聞きながら明日の調査について相談しました。朝になれば山沿いの集落まで歩き、図書館に残された写真と景色を照らし合わせる予定です。'


def control_plan(labels):
    if labels!=['arial','bold','serif']:raise ValueError('all three fixed font faces are required exactly once')
    if len(PARAGRAPH)<167 or any(c.isspace() for c in PARAGRAPH):raise ValueError('invalid fixed paragraph')
    rows=[]
    for font in labels:
        for length in [67,87,107,127,147,167]:
            for padding in [4,16]:rows.append({'id':f'r1_text_{font}_{length}_pad{padding}','cohort':'body','font':font,'text':PARAGRAPH[:length],'padding':padding})
        for index,pattern in enumerate(['空','春夏秋冬']):
            for padding in [4,16]:rows.append({'id':f'r1_repeat_{font}_{index}_pad{padding}','cohort':'repeated','font':font,'text':(pattern*87)[:87],'padding':padding})
        for index,pattern in enumerate(['港の地図','No.12']):
            for count in [30,44]:rows.append({'id':f'r1_portrait_{font}_{index}_{count}','cohort':'portrait','font':font,'text':'\n'.join([pattern]*count),'row_text':pattern,'rows':count})
    for background in ['white','warm','gray']:
        for pattern in ['open_slant_hexagon','offset_crown']:
            for groups in [72,132]:rows.append({'id':f'r1_shape_{background}_{pattern}_{groups}','cohort':'nonletter','text':'','background':background,'pattern':pattern,'groups':groups})
    return rows


def assert_font_covers(font,text):
    def signature(character):
        try:
            mask=font.getmask(character,mode='L');return mask.size,bytes(mask)
        except (UnicodeError,ValueError) as error:raise ValueError('font cannot render the declared scalar') from error
    missing=signature('\U0010ffff')
    for character in set(text):
        if character.isspace():continue
        value=signature(character)
        if value==missing or not any(value[1]):raise ValueError(f'missing glyph U+{ord(character):04X}')
